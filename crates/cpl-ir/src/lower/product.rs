//! Product domains in IR: a position encodes its parts' ordinals, row-major.

use super::*;

impl<'a> Lowerer<'a> {
    /// `P((r, c))`: a tuple of factor positions or integers, each integer
    /// checked into its factor (R04.2.10).
    pub(super) fn product_from_tuple(&mut self, id: DomainId, v: Expr, span: Span) -> Expr {
        let factors = self.product_domains[&id].clone();
        let elems = match &v.ty {
            Ty::Tuple(es) if es.len() == factors.len() && es.contains(&Ty::Int) => es.clone(),
            _ => return self.coerce(v, &Ty::Pos(id), span),
        };
        let mut out = Vec::new();
        let held = self.hold(v, &mut out);
        let mut parts = Vec::with_capacity(elems.len());
        for (i, ((_, f), ety)) in factors.iter().zip(elems).enumerate() {
            let part = self.mk(
                ExprKind::Field(Box::new(held.clone()), i),
                ety.clone(),
                span,
            );
            parts.push(match f {
                ProductFactor::Domain(d) if ety == Ty::Int => self.mk(
                    ExprKind::Cast(Cast::FromLabel(*d), Box::new(part)),
                    Ty::Pos(*d),
                    span,
                ),
                _ => {
                    let want = self.product_factor_ty(f);
                    self.coerce(part, &want, span)
                }
            });
        }
        let pos = self.encode_product(id, parts, span);
        out.push(Stmt::Give(pos));
        self.mk(ExprKind::Do(Block { stmts: out }), Ty::Pos(id), span)
    }

    /// `V.at(label)`: the position with that label, or `None` outside `V`.
    /// On a product, `P.at((x1, ..., xn))` takes the parts tuple, and is `None`
    /// when any part is outside its factor (R04.2.10).
    pub(super) fn domain_at(&mut self, id: DomainId, args: &[ast::Arg], span: Span) -> Expr {
        if self.product_domains.contains_key(&id) && args.len() == 1 && args[0].name.is_none() {
            let v = self.expr(&args[0].value, None);
            if matches!(v.ty, Ty::Tuple(_)) {
                return self.product_try_from_tuple(id, v, span);
            }
            let label = self.coerce(v, &Ty::Int, args[0].value.span);
            return self.domain_try_label(id, label, span);
        }
        match self.one_arg("at", args, &Ty::Int, span) {
            Some(label) => self.domain_try_label(id, label, span),
            None => self.err_expr(span),
        }
    }

    fn product_try_from_tuple(&mut self, id: DomainId, v: Expr, span: Span) -> Expr {
        let factors = self.product_domains[&id].clone();
        let want = Ty::opt(Ty::Pos(id));
        let elems = match &v.ty {
            Ty::Tuple(es) if es.len() == factors.len() => es.clone(),
            _ => return self.coerce(v, &want, span),
        };
        let mut out = Vec::new();
        let held = self.hold(v, &mut out);
        let mut parts = Vec::with_capacity(elems.len());
        let mut checks = Vec::new();
        for (i, ((_, f), ety)) in factors.iter().zip(elems).enumerate() {
            let part = self.mk(
                ExprKind::Field(Box::new(held.clone()), i),
                ety.clone(),
                span,
            );
            match f {
                ProductFactor::Domain(d) if ety == Ty::Int => {
                    let found = self.domain_try_label(*d, part, span);
                    let found = self.hold(found, &mut out);
                    let is_some = ExprKind::IsSome(Box::new(found.clone()));
                    checks.push(self.mk(is_some, Ty::Bool, span));
                    let unwrap = ExprKind::Unwrap(Box::new(found), false);
                    parts.push(self.mk(unwrap, Ty::Pos(*d), span));
                }
                _ => {
                    let ty = self.product_factor_ty(f);
                    parts.push(self.coerce(part, &ty, span));
                }
            }
        }
        let pos = self.encode_product(id, parts, span);
        let mut found = self.mk(ExprKind::Some(Box::new(pos)), want.clone(), span);
        for check in checks.into_iter().rev() {
            let none = self.mk(ExprKind::None, want.clone(), span);
            let pick = ExprKind::If(Box::new(check), Box::new(found), Box::new(none));
            found = self.mk(pick, want.clone(), span);
        }
        out.push(Stmt::Give(found));
        self.mk(ExprKind::Do(Block { stmts: out }), want, span)
    }
}
