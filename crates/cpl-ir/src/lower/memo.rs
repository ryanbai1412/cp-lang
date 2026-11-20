//! `@memo`: a fn that remembers its answers.
//!
//! The fn's body is lowered as it stands, and the fn itself becomes a
//! wrapper around it: look the arguments up, and call the body only when
//! they are new. The cache is ordinary IR, so the interpreter and the
//! C++ agree on it by construction.
//!
//! Where every argument is a position, a `Bool` or an `Int` of a
//! declared range, the cache is a flat table with one cell per point,
//! sized when the fn is first called. Past [`DENSE_LIMIT`] cells, or
//! for arguments with no bound (`w: Int`, a `Str`), it is a hash map.
//!
//! The cache of a top-level fn lasts for one test: the file's own lines
//! start it afresh. That of a nested fn belongs to the block the fn is
//! declared in, and is gone when the block is left.

use super::*;

/// The largest flat table, in cells.
const DENSE_LIMIT: i64 = 1 << 24;

/// Where the cache of a `@memo` fn is kept.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MemoHome {
    /// File-level variables.
    Global,
    /// Locals of the block that declares the fn, captured by the wrapper.
    Outer,
}

/// One coordinate of the flat table.
struct Part {
    /// Its 0-based offset.
    ord: Expr,
    len: Expr,
    /// The offset has to be checked: a declared range is not enforced
    /// at the call.
    ranged: bool,
}

struct Memo<'s> {
    home: MemoHome,
    id: usize,
    span: Span,
    /// Declarations for the enclosing block, ahead of the fn.
    lets: &'s mut Vec<Stmt>,
    /// Run once, before the first lookup.
    inits: Vec<Stmt>,
    parts: Vec<Part>,
}

fn read(place: &Place) -> Expr {
    let kind = match &place.kind {
        PlaceKind::Local(slot) => ExprKind::Local(*slot),
        PlaceKind::Global(id) => ExprKind::Global(*id),
        _ => unreachable!("a cache variable is a local or a global"),
    };
    Expr {
        kind,
        ty: place.ty.clone(),
        span: place.span,
    }
}

fn set(place: &Place, value: Expr) -> Stmt {
    Stmt::Assign(place.clone(), AssignOp::Set, value)
}

fn block(stmts: Vec<Stmt>) -> Block {
    Block { stmts }
}

impl<'a> Lowerer<'a> {
    /// Whether values of `ty` can key a hash map in both back ends.
    fn memo_hashable(ty: &Ty) -> bool {
        match ty {
            Ty::Int | Ty::Bool | Ty::Char | Ty::Str | Ty::Pos(_) | Ty::Bound(_) => true,
            Ty::Opt(inner) => Self::memo_hashable(inner),
            Ty::Tuple(items) => items.iter().all(Self::memo_hashable),
            Ty::Record(fields) => fields.iter().all(|(_, t)| Self::memo_hashable(t)),
            _ => false,
        }
    }

    /// A variable of the cache.
    fn memo_var(&mut self, memo: &mut Memo, tag: &str, ty: Ty) -> Place {
        let name = format!("__memo{}_{tag}", memo.id);
        let kind = match memo.home {
            MemoHome::Global => {
                self.globals.push(Global {
                    name,
                    ty: ty.clone(),
                });
                PlaceKind::Global(self.globals.len() - 1)
            }
            MemoHome::Outer => {
                let outer = self.outer_cxs.last_mut().expect("a nested fn has an enclosing one");
                let slot = outer.locals.len();
                outer.locals.push(Local {
                    name: name.clone(),
                    ty: ty.clone(),
                });
                outer.scopes.last_mut().expect("an open scope").insert(name.clone(), slot);
                outer.mut_slots.insert(slot);
                memo.lets.push(Stmt::Let(slot, None));
                PlaceKind::Local(self.capture_from_outer(&name).expect("just declared"))
            }
        };
        Place {
            kind,
            ty,
            span: self.span(memo.span),
        }
    }

    fn memo_cmp(&self, op: CmpOp, a: Expr, b: Expr, span: Span) -> Expr {
        self.mk(ExprKind::Cmp(op, Box::new(a), Box::new(b)), Ty::Bool, span)
    }

    fn memo_bin(&self, op: BinOp, a: Expr, b: Expr, span: Span) -> Expr {
        let ty = a.ty.clone();
        self.mk(ExprKind::Binary(op, Box::new(a), Box::new(b)), ty, span)
    }

    /// Splits `value` into coordinates of the flat table. `false` when
    /// some part of it has no bound.
    fn memo_parts(&mut self, memo: &mut Memo, value: Expr, spec: &Spec) -> bool {
        let span = memo.span;
        match spec {
            Spec::Plain(Ty::Pos(d)) | Spec::Plain(Ty::Bound(d)) => {
                let cells = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                let len = if matches!(spec, Spec::Plain(Ty::Bound(_))) {
                    self.int_add(cells, 1)
                } else {
                    cells
                };
                let ord = self.mk(ExprKind::Cast(Cast::Ord, Box::new(value)), Ty::Int, span);
                memo.parts.push(Part {
                    ord,
                    len,
                    ranged: false,
                });
                true
            }
            Spec::Plain(Ty::Bool) => {
                let (one, zero) = (self.int_lit(1, span), self.int_lit(0, span));
                let ord = self.mk(
                    ExprKind::If(Box::new(value), Box::new(one), Box::new(zero)),
                    Ty::Int,
                    span,
                );
                memo.parts.push(Part {
                    ord,
                    len: self.int_lit(2, span),
                    ranged: false,
                });
                true
            }
            Spec::Int {
                lo: Some(lo),
                hi: Some(hi),
                ..
            } => {
                let at = memo.parts.len();
                let first = self.memo_var(memo, &format!("lo{at}"), Ty::Int);
                let len = self.memo_var(memo, &format!("len{at}"), Ty::Int);
                memo.inits.push(set(&first, (**lo).clone()));
                let width = self.memo_bin(BinOp::Sub, (**hi).clone(), read(&first), span);
                memo.inits.push(set(&len, self.int_add(width, 1)));
                memo.parts.push(Part {
                    ord: self.memo_bin(BinOp::Sub, value, read(&first), span),
                    len: read(&len),
                    ranged: true,
                });
                true
            }
            Spec::Fields(ty @ (Ty::Tuple(_) | Ty::Record(_)), specs) => {
                let _ = ty;
                specs.iter().enumerate().all(|(i, field)| {
                    let part = self.mk(ExprKind::Field(Box::new(value.clone()), i), field.ty(), span);
                    self.memo_parts(memo, part, field)
                })
            }
            _ => false,
        }
    }

    /// The body of the wrapper of a `@memo` fn: `params` are its own
    /// parameters, as slots of the current fn, with their types as
    /// written, and `call` runs the fn's real body on them.
    pub(super) fn memo_wrapper(
        &mut self,
        home: MemoHome,
        params: &[(Slot, Option<&ast::Type>)],
        ret: &Ty,
        call: Expr,
        span: Span,
        lets: &mut Vec<Stmt>,
    ) -> Vec<Stmt> {
        self.memo_count += 1;
        let mut memo = Memo {
            home,
            id: self.memo_count,
            span,
            lets,
            inits: Vec::new(),
            parts: Vec::new(),
        };
        let args: Vec<Expr> = params.iter().map(|(slot, _)| self.local(*slot, span)).collect();
        let mut dense = !params.is_empty();
        for ((_, written), arg) in params.iter().zip(&args) {
            let spec = match written {
                Some(ty) => self.resolve(ty, true),
                None => Spec::Plain(arg.ty.clone()),
            };
            dense = dense && self.memo_parts(&mut memo, arg.clone(), &spec);
        }
        let key = match args.len() {
            1 => args[0].clone(),
            _ => {
                let ty = Ty::Tuple(args.iter().map(|a| a.ty.clone()).collect());
                self.mk(ExprKind::Tuple(args.clone()), ty, span)
            }
        };
        let hashed = Self::memo_hashable(&key.ty);
        if !dense && !hashed {
            let shown = self.show(&key.ty);
            self.unsupported(span, &format!("`@memo` on a fn of `{shown}`"));
            return vec![Stmt::Return(Some(call))];
        }

        let ready = self.memo_var(&mut memo, "ready", Ty::Bool);
        if home == MemoHome::Global {
            self.memo_resets.push(ready.clone());
        }
        let yes = self.mk(ExprKind::Bool(true), Ty::Bool, span);
        let no = self.mk(ExprKind::Bool(false), Ty::Bool, span);
        let mut start = vec![set(&ready, yes.clone())];
        start.append(&mut memo.inits);
        let mut out = Vec::new();

        // The call, and what to do with its answer.
        let answer = |lw: &mut Self, keep: &dyn Fn(Expr) -> Vec<Stmt>| {
            let slot = lw.temp(ret.clone());
            let mut stmts = vec![Stmt::Let(slot, Some(call.clone()))];
            stmts.extend(keep(lw.local(slot, span)));
            stmts.push(Stmt::Return(Some(lw.local(slot, span))));
            stmts
        };

        if dense {
            let flat = self.memo_var(&mut memo, "flat", Ty::Bool);
            let seen = self.memo_var(&mut memo, "seen", Ty::list(Ty::Bool));
            let vals = self.memo_var(&mut memo, "vals", Ty::list(ret.clone()));
            let cells = self.temp(Ty::Int);
            start.push(Stmt::Let(cells, Some(self.int_lit(1, span))));
            start.push(set(&flat, yes.clone()));
            let cells_place = Place {
                kind: PlaceKind::Local(cells),
                ty: Ty::Int,
                span: self.span(span),
            };
            for part in &memo.parts {
                // cells * len <= DENSE_LIMIT, without the product.
                let room = self.memo_bin(
                    BinOp::Div,
                    self.int_lit(DENSE_LIMIT, span),
                    self.local(cells, span),
                    span,
                );
                let some = self.memo_cmp(CmpOp::Ge, part.len.clone(), self.int_lit(1, span), span);
                let fits = self.memo_cmp(CmpOp::Le, part.len.clone(), room, span);
                let fits = self.memo_bin(BinOp::And, some, fits, span);
                let fits = self.memo_bin(BinOp::And, read(&flat), fits, span);
                let grown = self.memo_bin(BinOp::Mul, self.local(cells, span), part.len.clone(), span);
                start.push(Stmt::If(
                    fits,
                    block(vec![set(&cells_place, grown)]),
                    block(vec![set(&flat, no.clone())]),
                ));
            }
            let zero = self.temp(ret.clone());
            let filled = |lw: &Self, value: Expr| {
                let ty = Ty::list(value.ty.clone());
                lw.mk(
                    ExprKind::Builtin(Builtin::ListFilled, vec![lw.local(cells, span), value]),
                    ty,
                    span,
                )
            };
            start.push(Stmt::If(
                read(&flat),
                block(vec![
                    Stmt::Let(zero, None),
                    set(&seen, filled(self, no.clone())),
                    set(&vals, filled(self, self.local(zero, span))),
                ]),
                Block::default(),
            ));

            let mut inside = read(&flat);
            let mut index = self.int_lit(0, span);
            for part in &memo.parts {
                if part.ranged {
                    let low = self.memo_cmp(CmpOp::Ge, part.ord.clone(), self.int_lit(0, span), span);
                    let high = self.memo_cmp(CmpOp::Lt, part.ord.clone(), part.len.clone(), span);
                    inside = self.memo_bin(BinOp::And, inside, low, span);
                    inside = self.memo_bin(BinOp::And, inside, high, span);
                }
                let scaled = self.memo_bin(BinOp::Mul, index, part.len.clone(), span);
                index = self.memo_bin(BinOp::Add, scaled, part.ord.clone(), span);
            }
            let at = self.temp(Ty::Int);
            let cell = |lw: &Self, of: &Place, ty: Ty| Place {
                kind: PlaceKind::Index(Box::new(of.clone()), vec![lw.local(at, span)]),
                ty,
                span: lw.span(span),
            };
            let cell_of = |lw: &Self, of: &Place, ty: Ty| {
                lw.mk(
                    ExprKind::Index(Box::new(read(of)), vec![lw.local(at, span)]),
                    ty,
                    span,
                )
            };
            let mut found = vec![Stmt::Let(at, Some(index))];
            found.push(Stmt::If(
                cell_of(self, &seen, Ty::Bool),
                block(vec![Stmt::Return(Some(cell_of(self, &vals, ret.clone())))]),
                Block::default(),
            ));
            let (seen_cell, vals_cell) = (cell(self, &seen, Ty::Bool), cell(self, &vals, ret.clone()));
            found.extend(answer(self, &|value| {
                vec![set(&vals_cell, value), set(&seen_cell, yes.clone())]
            }));
            out.push(Stmt::If(inside, block(found), Block::default()));
        }

        if hashed {
            let table_ty = Ty::HashMap(Box::new(key.ty.clone()), Box::new(ret.clone()));
            let table = self.memo_var(&mut memo, "table", table_ty.clone());
            let empty = self.mk(ExprKind::Tuple(Vec::new()), table_ty, span);
            start.push(set(&table, empty));
            let k = self.temp(key.ty.clone());
            out.push(Stmt::Let(k, Some(key)));
            let known = self.mk(
                ExprKind::Builtin(Builtin::Contains, vec![read(&table), self.local(k, span)]),
                Ty::Bool,
                span,
            );
            let stored = self.mk(
                ExprKind::Index(Box::new(read(&table)), vec![self.local(k, span)]),
                ret.clone(),
                span,
            );
            out.push(Stmt::If(
                known,
                block(vec![Stmt::Return(Some(stored))]),
                Block::default(),
            ));
            let entry = Place {
                kind: PlaceKind::Index(Box::new(table.clone()), vec![self.local(k, span)]),
                ty: ret.clone(),
                span: self.span(span),
            };
            out.extend(answer(self, &|value| vec![set(&entry, value)]));
        } else {
            // Outside the table: an argument past its declared range.
            out.push(Stmt::Return(Some(call.clone())));
        }

        let fresh = self.mk(ExprKind::Unary(UnOp::Not, Box::new(read(&ready))), Ty::Bool, span);
        out.insert(0, Stmt::If(fresh, block(start), Block::default()));
        out
    }
}
