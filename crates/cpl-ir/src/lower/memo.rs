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
//! for arguments with no bound (a `Str`), it is a hash map.
//!
//! A fn of one `Int` too wide for a table (`w: 1..=1e9`, `w: Int`) gets
//! both: a recursion on a number asks for small ones far more often
//! than large ones, so the first [`PREFIX`] values have cells, made as
//! they are reached, and the rest are hashed.
//!
//! The cache of a top-level fn lasts for one test: the file's own lines
//! start it afresh. That of a nested fn belongs to the block the fn is
//! declared in, and is gone when the block is left.

use super::*;

/// The largest flat table, in cells.
const DENSE_LIMIT: i64 = 1 << 24;

/// How many of the smallest values of a wide `Int` get a cell.
const PREFIX: i64 = 1 << 20;

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
    /// The least and greatest `len`, when the declarations bound it.
    most: Option<(i64, i64)>,
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

/// The lookup in a flat table, kept apart from the code that fills it.
pub(super) struct MemoFast {
    /// The table is made, is flat, and the arguments are within it.
    inside: Expr,
    index: Expr,
    seen: Place,
    vals: Place,
}

/// The table a `@memo(into: dp)` fn keeps its answers in.
pub(super) fn into_table(stmt: &ast::Stmt) -> Option<&str> {
    let memo = stmt.decorators.iter().find(|d| d.name.text == "memo")?;
    memo.args.iter().find_map(|arg| match (&arg.name, &arg.value.kind) {
        (Some(label), ast::ExprKind::Name(table)) if label.text == "into" => Some(table.as_str()),
        _ => None,
    })
}

/// The name of a local the wrapper of a `@memo(into:)` fn makes. No
/// program can write a name with a space in it.
const HIT: &str = "memo hit";
const ANSWER: &str = "memo answer";

fn name_expr(text: &str, span: Span) -> ast::Expr {
    ast::Expr {
        kind: ast::ExprKind::Name(text.to_string()),
        span,
    }
}

fn plain_stmt(kind: ast::StmtKind, span: Span) -> ast::Stmt {
    ast::Stmt {
        kind,
        span,
        decorators: Vec::new(),
        public: false,
    }
}

/// `dp[i, j]` for the fn's own parameters.
fn into_cell(table: &str, decl: &ast::FnDecl) -> ast::Expr {
    let span = decl.name.span;
    ast::Expr {
        kind: ast::ExprKind::Index {
            base: Box::new(name_expr(table, span)),
            indices: decl.params.iter().map(|p| name_expr(&p.name.text, span)).collect(),
        },
        span,
    }
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
                    span: Span::default(),
                });
                PlaceKind::Global(self.globals.len() - 1)
            }
            MemoHome::Outer => {
                let outer = self
                    .outer_cxs
                    .last_mut()
                    .expect("a nested fn has an enclosing one");
                let slot = outer.locals.len();
                outer.locals.push(Local {
                    name: name.clone(),
                    ty: ty.clone(),
                });
                outer
                    .scopes
                    .last_mut()
                    .expect("an open scope")
                    .insert(name.clone(), slot);
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

    /// `@memo(into: dp)` where the fn stands: `var dp: [I, J]?T = none`,
    /// one cell for each choice of arguments.
    pub(super) fn memo_into_table(&mut self, table: &str, decl: &ast::FnDecl, out: &mut Vec<Stmt>) {
        let span = decl.name.span;
        let index: Option<Vec<ast::Type>> = decl.params.iter().map(|p| p.ty.clone()).collect();
        let (Some(index), Some(ret)) = (index, decl.ret.clone()) else {
            self.unsupported(span, "`@memo(into:)` on a fn whose parameter and return types are not all written");
            return;
        };
        // A table of `0..=9` answers holds `Int`s.
        let elem = match ret.kind {
            ast::TypeKind::Range { .. } => ast::Type {
                kind: ast::TypeKind::Named {
                    name: ast::Name {
                        text: "Int".to_string(),
                        span: ret.span,
                    },
                    args: Vec::new(),
                },
                span: ret.span,
            },
            _ => ret,
        };
        let ty = ast::Type {
            kind: ast::TypeKind::Array {
                index,
                elem: Box::new(ast::Type {
                    span: elem.span,
                    kind: ast::TypeKind::Optional(Box::new(elem)),
                }),
            },
            span,
        };
        let var = ast::StmtKind::Let {
            mutable: true,
            pat: ast::Pattern {
                kind: ast::PatternKind::Name(table.to_string()),
                span,
            },
            ty: Some(ty),
            init: Some(ast::Expr {
                kind: ast::ExprKind::None,
                span,
            }),
            otherwise: None,
            guard: None,
        };
        self.stmt(&plain_stmt(var, span), out);
    }

    /// The wrapper of a `@memo(into: dp)` fn: the answer in `dp` if it
    /// is there, and otherwise `fill`, kept in `dp`. The table is the
    /// program's to read, fill ahead of a call, or clear.
    pub(super) fn memo_into_wrapper(
        &mut self,
        table: &str,
        decl: &ast::FnDecl,
        fill: Expr,
        ret: &Ty,
    ) -> Vec<Stmt> {
        let span = decl.name.span;
        let cell = into_cell(table, decl);
        let mut out = Vec::new();
        let give = plain_stmt(ast::StmtKind::Return(Some(name_expr(HIT, span))), span);
        let hit = ast::Expr {
            kind: ast::ExprKind::If {
                branches: vec![(
                    ast::Cond::Let {
                        pat: ast::Pattern {
                            kind: ast::PatternKind::Name(HIT.to_string()),
                            span,
                        },
                        value: cell.clone(),
                        and: None,
                    },
                    ast::Block {
                        stmts: vec![give],
                        span,
                    },
                )],
                otherwise: None,
            },
            span,
        };
        self.stmt(&plain_stmt(ast::StmtKind::Expr(hit), span), &mut out);
        let answer = self.cx.locals.len();
        self.cx.scopes[0].insert(ANSWER.to_string(), answer);
        self.cx.locals.push(Local {
            name: ANSWER.to_string(),
            ty: ret.clone(),
        });
        out.push(Stmt::Let(answer, Some(fill)));
        let keep = ast::StmtKind::Assign {
            target: cell,
            op: ast::AssignOp::Set,
            value: name_expr(ANSWER, span),
        };
        self.stmt(&plain_stmt(keep, span), &mut out);
        out.push(Stmt::Return(Some(self.local(answer, span))));
        out
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
                let bound = i64::from(matches!(spec, Spec::Plain(Ty::Bound(_))));
                let most = self.domain_lens.get(d).map(|&(least, most)| (least + bound, most + bound));
                let ord = self.mk(ExprKind::Cast(Cast::Ord, Box::new(value)), Ty::Int, span);
                memo.parts.push(Part {
                    ord,
                    len,
                    ranged: false,
                    most,
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
                    most: Some((2, 2)),
                });
                true
            }
            Spec::Int { .. } | Spec::Plain(Ty::Int) => {
                let (lo, hi) = match spec {
                    Spec::Int { lo, hi, .. } => (lo.as_deref().cloned(), hi.as_deref().cloned()),
                    _ => (None, None),
                };
                let at = memo.parts.len();
                let first = self.memo_var(memo, &format!("lo{at}"), Ty::Int);
                let len = self.memo_var(memo, &format!("len{at}"), Ty::Int);
                let lo = lo.unwrap_or_else(|| self.int_lit(0, span));
                let most = match (self.ir_bounds(&lo), hi.as_ref().and_then(|hi| self.ir_bounds(hi))) {
                    (Some(l), Some(h)) => h.0.checked_sub(l.1).zip(h.1.checked_sub(l.0)),
                    _ => None,
                }
                .and_then(|(least, most)| least.checked_add(1).zip(most.checked_add(1)));
                memo.inits.push(set(&first, lo));
                // With no upper bound: wider than any table.
                let size = match hi {
                    Some(hi) => {
                        let width = self.memo_bin(BinOp::Sub, hi, read(&first), span);
                        self.int_add(width, 1)
                    }
                    None => self.int_lit(DENSE_LIMIT + 1, span),
                };
                memo.inits.push(set(&len, size));
                memo.parts.push(Part {
                    ord: self.memo_bin(BinOp::Sub, value, read(&first), span),
                    len: read(&len),
                    ranged: true,
                    most,
                });
                true
            }
            Spec::Fields(ty @ (Ty::Tuple(_) | Ty::Record(_)), specs) => {
                let _ = ty;
                specs.iter().enumerate().all(|(i, field)| {
                    let part = self.mk(
                        ExprKind::Field(Box::new(value.clone()), i),
                        field.ty(),
                        span,
                    );
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
        let args: Vec<Expr> = params
            .iter()
            .map(|(slot, _)| self.local(*slot, span))
            .collect();
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
        // When the declarations prove the table small, it is all there
        // is: no hash map beside it and no test of its size.
        let proven = dense
            && memo
                .parts
                .iter()
                .try_fold(1i64, |cells, part| match part.most {
                    Some((least, most)) if least >= 1 => cells.checked_mul(most),
                    _ => None,
                })
                .is_some_and(|cells| cells <= DENSE_LIMIT);
        let hashed = !proven && Self::memo_hashable(&key.ty);
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
            let flat = (!proven).then(|| self.memo_var(&mut memo, "flat", Ty::Bool));
            let seen = self.memo_var(&mut memo, "seen", Ty::list(Ty::Bool));
            let vals = self.memo_var(&mut memo, "vals", Ty::list(ret.clone()));
            let cells = self.temp(Ty::Int);
            start.push(Stmt::Let(cells, Some(self.int_lit(1, span))));
            if let Some(flat) = &flat {
                start.push(set(flat, yes.clone()));
            }
            let cells_place = Place {
                kind: PlaceKind::Local(cells),
                ty: Ty::Int,
                span: self.span(span),
            };
            // One wide `Int`: cells for its smallest values only, made
            // as they are reached.
            let lazy = !proven
                && matches!(
                &memo.parts[..],
                [Part {
                    ranged: true,
                    len: Expr {
                        kind: ExprKind::Local(_) | ExprKind::Global(_),
                        ..
                    },
                    ..
                }]
            );
            for part in &memo.parts {
                let grown =
                    self.memo_bin(BinOp::Mul, self.local(cells, span), part.len.clone(), span);
                let Some(flat) = &flat else {
                    start.push(set(&cells_place, grown));
                    continue;
                };
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
                let fits = self.memo_bin(BinOp::And, read(flat), fits, span);
                let wide = if lazy {
                    let kind = match &part.len.kind {
                        ExprKind::Local(slot) => PlaceKind::Local(*slot),
                        ExprKind::Global(id) => PlaceKind::Global(*id),
                        _ => unreachable!("checked above"),
                    };
                    let len = Place {
                        kind,
                        ty: Ty::Int,
                        span: self.span(span),
                    };
                    let positive =
                        self.memo_cmp(CmpOp::Ge, part.len.clone(), self.int_lit(1, span), span);
                    vec![Stmt::If(
                        positive,
                        block(vec![
                            set(&len, self.int_lit(PREFIX, span)),
                            set(&cells_place, self.int_lit(0, span)),
                        ]),
                        block(vec![set(flat, no.clone())]),
                    )]
                } else {
                    vec![set(flat, no.clone())]
                };
                start.push(Stmt::If(
                    fits,
                    block(vec![set(&cells_place, grown)]),
                    block(wide),
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
            let make = vec![
                Stmt::Let(zero, None),
                set(&seen, filled(self, no.clone())),
                set(&vals, filled(self, self.local(zero, span))),
            ];
            match &flat {
                Some(flat) => start.push(Stmt::If(read(flat), block(make), Block::default())),
                None => start.extend(make),
            }

            let mut inside = match &flat {
                Some(flat) => read(flat),
                None => yes.clone(),
            };
            let mut index = self.int_lit(0, span);
            for part in &memo.parts {
                if part.ranged {
                    let low =
                        self.memo_cmp(CmpOp::Ge, part.ord.clone(), self.int_lit(0, span), span);
                    let high = self.memo_cmp(CmpOp::Lt, part.ord.clone(), part.len.clone(), span);
                    inside = self.memo_bin(BinOp::And, inside, low, span);
                    inside = self.memo_bin(BinOp::And, inside, high, span);
                }
                let scaled = self.memo_bin(BinOp::Mul, index, part.len.clone(), span);
                index = self.memo_bin(BinOp::Add, scaled, part.ord.clone(), span);
            }
            if home == MemoHome::Global {
                for table in [&seen, &vals] {
                    if let PlaceKind::Global(id) = table.kind {
                        self.raw_lists.push(id);
                    }
                }
                if !lazy {
                    self.memo_fast = Some(MemoFast {
                        inside: self.memo_bin(BinOp::And, read(&ready), inside.clone(), span),
                        index: index.clone(),
                        seen: seen.clone(),
                        vals: vals.clone(),
                    });
                }
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
            if lazy {
                let have = |lw: &Self| {
                    lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![read(&seen)]),
                        Ty::Int,
                        span,
                    )
                };
                let short = self.memo_cmp(CmpOp::Ge, self.local(at, span), have(self), span);
                let twice = self.memo_bin(BinOp::Mul, have(self), self.int_lit(2, span), span);
                let need = self.int_add(self.local(at, span), 1);
                let more = self.mk(
                    ExprKind::Builtin(Builtin::Max, vec![twice, need]),
                    Ty::Int,
                    span,
                );
                let size = self.mk(
                    ExprKind::Builtin(Builtin::Min, vec![more, memo.parts[0].len.clone()]),
                    Ty::Int,
                    span,
                );
                let to = self.temp(Ty::Int);
                let blank = self.temp(ret.clone());
                let resize = |lw: &Self, list: &Place, fill: Expr| {
                    Stmt::Expr(lw.mk(
                        ExprKind::Mutate(
                            Mutation::Resize,
                            Box::new(list.clone()),
                            vec![lw.local(to, span), fill],
                        ),
                        Ty::Unit,
                        span,
                    ))
                };
                found.push(Stmt::If(
                    short,
                    block(vec![
                        Stmt::Let(to, Some(size)),
                        Stmt::Let(blank, None),
                        resize(self, &seen, no.clone()),
                        resize(self, &vals, self.local(blank, span)),
                    ]),
                    Block::default(),
                ));
            }
            found.push(Stmt::If(
                cell_of(self, &seen, Ty::Bool),
                block(vec![Stmt::Return(Some(cell_of(self, &vals, ret.clone())))]),
                Block::default(),
            ));
            let (seen_cell, vals_cell) =
                (cell(self, &seen, Ty::Bool), cell(self, &vals, ret.clone()));
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
            let got_ty = Ty::Opt(Box::new(ret.clone()));
            let got = self.temp(got_ty.clone());
            let lookup = self.mk(
                ExprKind::Builtin(Builtin::MapGet, vec![read(&table), self.local(k, span)]),
                got_ty.clone(),
                span,
            );
            out.push(Stmt::Let(got, Some(lookup)));
            let known = self.mk(
                ExprKind::IsSome(Box::new(self.local(got, span))),
                Ty::Bool,
                span,
            );
            let stored = self.mk(
                ExprKind::Unwrap(Box::new(self.local(got, span)), false),
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

        let fresh = self.mk(
            ExprKind::Unary(UnOp::Not, Box::new(read(&ready))),
            Ty::Bool,
            span,
        );
        out.insert(0, Stmt::If(fresh, block(start), Block::default()));
        out
    }

    /// The wrapper of a `@memo` fn with a flat table: a cell that is
    /// already filled is answered here, and anything else by `fill`.
    pub(super) fn memo_lookup(&mut self, fast: MemoFast, fill: Expr, ret: &Ty, span: Span) -> Vec<Stmt> {
        let at = self.temp(Ty::Int);
        let cell = |lw: &Self, of: &Place, ty: Ty| {
            lw.mk(
                ExprKind::Index(Box::new(read(of)), vec![lw.local(at, span)]),
                ty,
                span,
            )
        };
        let known = cell(self, &fast.seen, Ty::Bool);
        let stored = cell(self, &fast.vals, ret.clone());
        vec![
            Stmt::If(
                fast.inside,
                block(vec![
                    Stmt::Let(at, Some(fast.index)),
                    Stmt::If(
                        known,
                        block(vec![Stmt::Return(Some(stored))]),
                        Block::default(),
                    ),
                ]),
                Block::default(),
            ),
            Stmt::Return(Some(fill)),
        ]
    }
}
