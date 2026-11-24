//! An array value meeting a declared `[D]T` whose index is a count
//! (`[0..n]Int`, `[1..=n]Int`): the value must have the keys the type
//! declares, at a `let`, a parameter, a field, a `return` and an `=`. The
//! checker rejects two counts it knows differ (E0302); a count known only
//! as the program runs is checked then, in every build, because a wrong
//! count is a type mismatch, not a debug check.

use super::*;

const WRONG_KEYS: &str = "this array doesn't have the keys its declared type gives it";

impl<'a> Lowerer<'a> {
    pub(super) fn declared_value(
        &mut self,
        ty: Option<&ast::Type>,
        init: Option<&ast::Expr>,
        span: Span,
    ) -> Option<Expr> {
        let depth = self.written.len();
        let value = self.declared_value_inner(ty, init, span);
        self.written.truncate(depth);
        value
    }

    /// `want`, the declared type `ty` names; while its value is lowered, a
    /// counted array is shown as written, `[n]Int` rather than `[Int]Int`.
    pub(super) fn written_as(&mut self, want: Ty, ty: &ast::Type) -> Ty {
        let counted = matches!(&want, Ty::Array { dims, .. } if dims.iter().any(|d| matches!(d, Dim::Int | Dim::IntFrom(_))));
        if counted {
            self.written.push((want.clone(), self.source(ty.span)));
        }
        want
    }

    /// The count each count-indexed axis of `ty` declares, by axis.
    fn declared_counts(&mut self, ty: &ast::Type) -> Vec<(usize, Expr)> {
        let ast::TypeKind::Array { index, .. } = &ty.kind else {
            return Vec::new();
        };
        let saved = self.diags.len();
        let mut counts = Vec::new();
        let mut axis = 0;
        for d in index {
            if matches!(d.kind, ast::TypeKind::Named { .. }) {
                axis += self.product_dims(d).map_or(1, |parts| parts.len());
                continue;
            }
            if let Some(DimSpec::Int(len) | DimSpec::IntFrom(_, len)) = self.dim(d, true) {
                counts.push((axis, *len));
            }
            axis += 1;
        }
        self.diags.truncate(saved);
        counts
    }

    /// Traps unless each counted axis of `held` has its count.
    fn push_count_checks(
        &mut self,
        held: &Expr,
        counts: Vec<(usize, Expr)>,
        span: Span,
        out: &mut Vec<Stmt>,
    ) {
        let Ty::Array { dims, .. } = &held.ty else {
            return;
        };
        let rank = dims.len();
        for (axis, want) in counts {
            if !matches!(dims.get(axis), Some(Dim::Int | Dim::IntFrom(_))) {
                continue;
            }
            let have = if rank == 1 {
                self.mk(
                    ExprKind::Builtin(Builtin::Len, vec![held.clone()]),
                    Ty::Int,
                    span,
                )
            } else {
                let at = self.int_lit(axis as i64, span);
                let args = vec![held.clone(), at];
                self.mk(ExprKind::Builtin(Builtin::AxisLen, args), Ty::Int, span)
            };
            let wrong = self.mk(
                ExprKind::Cmp(CmpOp::Ne, Box::new(have), Box::new(want)),
                Ty::Bool,
                span,
            );
            let trap = Stmt::Trap(WRONG_KEYS.to_string(), self.span(span));
            out.push(Stmt::If(
                wrong,
                Block { stmts: vec![trap] },
                Block::default(),
            ));
        }
    }

    fn counted(&mut self, value: Expr, counts: Vec<(usize, Expr)>, span: Span) -> Expr {
        if counts.is_empty()
            || !matches!(value.ty, Ty::Array { .. })
            || matches!(value.kind, ExprKind::Fill(..))
        {
            return value;
        }
        let want = value.ty.clone();
        if names_storage(&value) {
            // Reading a name twice reads the same array: check it where it
            // is rather than copy it into a temporary first.
            return self.do_expr(Some(&want), span, |lw, out| {
                lw.push_count_checks(&value, counts, span, out);
                out.push(Stmt::Give(value));
            });
        }
        self.do_expr(Some(&want), span, |lw, out| {
            let slot = lw.temp(want.clone());
            out.push(Stmt::Let(slot, Some(value)));
            let held = lw.local(slot, span);
            lw.push_count_checks(&held, counts, span, out);
            out.push(Stmt::Give(held));
        })
    }

    /// The counts `init` doesn't already write as the same number, as
    /// `[0..2] b => f(b)` does for a `[0..2]T`.
    fn unwritten(&mut self, init: &ast::Expr, counts: Vec<(usize, Expr)>) -> Vec<(usize, Expr)> {
        let ast::ExprKind::Table { domains, .. } = &init.kind else {
            return counts;
        };
        let saved = self.diags.len();
        let left = counts
            .into_iter()
            .filter(|(axis, want)| {
                let ExprKind::Int(k) = want.kind else {
                    return true;
                };
                let own = domains.get(*axis).and_then(|d| self.dim(d, true));
                !matches!(own, Some(DimSpec::Int(len) | DimSpec::IntFrom(_, len))
                    if len.kind == ExprKind::Int(k))
            })
            .collect();
        self.diags.truncate(saved);
        left
    }

    /// `value`, the lowered `init`, checked to have the keys `ty` declares.
    fn with_declared_keys(&mut self, value: Expr, ty: &ast::Type, init: &ast::Expr) -> Expr {
        if !matches!(value.ty, Ty::Array { .. }) {
            return value;
        }
        let counts = self.declared_counts(ty);
        let counts = self.unwritten(init, counts);
        self.counted(value, counts, init.span)
    }

    /// `let x: T = init`, as [`Self::declared_value`] lowers it, with
    /// its keys checked when `init` is an array of its own: one the
    /// declared type shapes (`read()`, a fill, a list literal) has them.
    pub(super) fn let_value(
        &mut self,
        ty: Option<&ast::Type>,
        init: Option<&ast::Expr>,
        span: Span,
    ) -> Option<Expr> {
        let value = self.declared_value(ty, init, span)?;
        Some(match (ty, init) {
            (Some(ty), Some(init))
                if counted_type(ty)
                    && !matches!(init.kind, ast::ExprKind::List(_))
                    && matches!(self.probe_ty_silent(init), Some(Ty::Array { .. })) =>
            {
                self.with_declared_keys(value, ty, init)
            }
            _ => value,
        })
    }

    /// A struct's field `ty` given `value`: its counts read where the
    /// struct is declared.
    pub(super) fn field_keys(
        &mut self,
        id: StructId,
        ty: &ast::Type,
        value: Expr,
        arg: &ast::Expr,
    ) -> Expr {
        let (m, offset) = (self.structs[id].mod_id, self.structs[id].offset);
        let sm = std::mem::replace(&mut self.cur_mod, m);
        let so = std::mem::replace(&mut self.offset, offset);
        let counts = if matches!(value.ty, Ty::Array { .. }) {
            self.declared_counts(ty)
        } else {
            Vec::new()
        };
        let counts = self.unwritten(arg, counts);
        self.cur_mod = sm;
        self.offset = so;
        self.counted(value, counts, arg.span)
    }

    /// On entry to a fn: checks each parameter's keys, and reads the
    /// counts its return type declares, for [`Self::check_returned_keys`].
    /// The keys a function's parameters and return values must have:
    /// parameters are checked on entry, ahead of the lowered `body`.
    pub(super) fn fn_keys(&mut self, decl: &ast::FnDecl, body: &mut Vec<Stmt>) {
        let mut entry = Vec::new();
        let ret = self.entry_keys(decl, &mut entry);
        body.splice(0..0, entry);
        self.check_returned_keys(&ret, body);
    }

    fn entry_keys(&mut self, decl: &ast::FnDecl, out: &mut Vec<Stmt>) -> Vec<(usize, Expr)> {
        for p in &decl.params {
            let Some(ty) = &p.ty else { continue };
            let Some(&slot) = self.cx.scopes[0].get(&p.name.text) else {
                continue;
            };
            if !matches!(self.cx.locals[slot].ty, Ty::Array { .. }) {
                continue;
            }
            let counts = self.declared_counts(ty);
            let held = self.local(slot, ty.span);
            self.push_count_checks(&held, counts, ty.span, out);
        }
        let Some(ty) = &decl.ret else {
            return Vec::new();
        };
        if !matches!(self.cx.ret, Some(Ty::Array { .. })) {
            return Vec::new();
        }
        let counts = self.declared_counts(ty);
        counts
            .into_iter()
            .map(|(axis, len)| {
                let slot = self.temp(Ty::Int);
                out.push(Stmt::Let(slot, Some(len)));
                (axis, self.local(slot, ty.span))
            })
            .collect()
    }

    /// Checks every value the fn returns against `counts`.
    fn check_returned_keys(&mut self, counts: &[(usize, Expr)], stmts: &mut [Stmt]) {
        if counts.is_empty() {
            return;
        }
        for stmt in stmts {
            match stmt {
                Stmt::Return(Some(v)) => {
                    let span = v.span;
                    let value = std::mem::replace(v, self.err_expr(span));
                    *v = self.counted(value, counts.to_vec(), span);
                }
                Stmt::If(_, a, b) => {
                    self.check_returned_keys(counts, &mut a.stmts);
                    self.check_returned_keys(counts, &mut b.stmts);
                }
                Stmt::While(_, _, body) | Stmt::For(_, _, _, body) => {
                    self.check_returned_keys(counts, &mut body.stmts)
                }
                _ => {}
            }
        }
    }

    /// `place = value` for a whole array: `value` keeps the place's keys.
    pub(super) fn keys_like_place(
        &mut self,
        place: &Place,
        op: AssignOp,
        value: Expr,
        span: Span,
    ) -> Expr {
        let Ty::Array { dims, .. } = &place.ty else {
            return value;
        };
        if op != AssignOp::Set || !matches!(value.ty, Ty::Array { .. }) {
            return value;
        }
        let rank = dims.len();
        let current = super::stmt::place_to_expr(place);
        let counts: Vec<(usize, Expr)> = (0..rank)
            .filter(|&axis| matches!(dims[axis], Dim::Int | Dim::IntFrom(_)))
            .map(|axis| {
                let have = if rank == 1 {
                    ExprKind::Builtin(Builtin::Len, vec![current.clone()])
                } else {
                    let at = self.int_lit(axis as i64, span);
                    ExprKind::Builtin(Builtin::AxisLen, vec![current.clone(), at])
                };
                (axis, self.mk(have, Ty::Int, span))
            })
            .collect();
        self.counted(value, counts, span)
    }
}

/// Whether `ty` is an array with an index written as a count, not a name.
fn counted_type(ty: &ast::Type) -> bool {
    matches!(&ty.kind, ast::TypeKind::Array { index, .. }
        if index.iter().any(|d| !matches!(d.kind, ast::TypeKind::Named { .. })))
}

/// A local, a global or a field of one: reading it runs nothing and
/// can't trap.
fn names_storage(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Local(_) | ExprKind::Global(_) => true,
        ExprKind::Field(base, _) => names_storage(base),
        _ => false,
    }
}
