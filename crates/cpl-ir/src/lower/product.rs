//! Product domains in IR: a position encodes its parts' ordinals, row-major.

use super::*;

impl<'a> Lowerer<'a> {
    /// Every position of `d`, as a range: `hi` is one past the last key and
    /// `lo` is no key at all when the domain is empty.
    pub(super) fn domain_keys(&mut self, d: DomainId, span: Span) -> Expr {
        let lo = self.mk(
            ExprKind::Cast(Cast::Trust, Box::new(self.int_lit(0, span))),
            Ty::Pos(d),
            span,
        );
        let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
        let hi = self.mk(ExprKind::Cast(Cast::Trust, Box::new(len)), Ty::Pos(d), span);
        self.mk(
            ExprKind::Range {
                lo: Box::new(lo),
                hi: Box::new(hi),
            },
            Ty::Range(Box::new(Ty::Pos(d))),
            span,
        )
    }

    /// `(R, C)` naming factor domains: the factors of that inline product.
    fn inline_product_factors(
        &self,
        e: &ast::Expr,
    ) -> Option<Vec<(Option<String>, ProductFactor)>> {
        let ast::ExprKind::Tuple(items) = &e.kind else {
            return None;
        };
        if items.len() < 2 {
            return None;
        }
        items
            .iter()
            .map(|it| self.expr_to_product_factor(it).map(|f| (None, f)))
            .collect()
    }

    /// The inline product domain `(R, C)` names, once interned (R04.2.20).
    pub(super) fn inline_product_sym(&self, e: &ast::Expr) -> Option<Sym> {
        let factors = self.inline_product_factors(e)?;
        self.anon_product_domains
            .get(&factors)
            .map(|&d| Sym::Domain(d))
    }

    /// Intern and define the inline product `e` names, so `(R, C).count(…)`
    /// reads it as a domain, like a named `domain P = (R, C)` (R04.2.20).
    pub(super) fn intern_inline_product(&mut self, e: &ast::Expr, span: Span) -> Option<DomainId> {
        let factors = self.inline_product_factors(e)?;
        let d = self.intern_anon_product_domain(factors);
        let mut defs = Vec::new();
        self.def_anon_product_domain(d, span, &mut defs);
        self.cx.pending.extend(defs);
        Some(d)
    }

    /// `(R, C)` in value position: the inline product domain over those
    /// factors, a domain value. A wanted tuple keeps it a tuple of domains.
    pub(super) fn inline_product_value(
        &mut self,
        e: &ast::Expr,
        want: Option<&Ty>,
    ) -> Option<Expr> {
        if matches!(want, Some(Ty::Tuple(_) | Ty::Record(_))) {
            return None;
        }
        let d = self.intern_inline_product(e, e.span)?;
        Some(self.mk(ExprKind::DomainVal(d), Ty::DomainVal, e.span))
    }

    /// R04.2.10: `type Cell = (Row, Col)` names the product of its factor domains.
    pub(super) fn alias_factors(
        &mut self,
        name: &str,
    ) -> Option<Vec<(Option<String>, ProductFactor)>> {
        let Some(Sym::Alias(aid)) = self.get_sym(name) else {
            return None;
        };
        let (m, ty_ast, offset) = self.aliases[aid];
        let sm = std::mem::replace(&mut self.cur_mod, m);
        let so = std::mem::replace(&mut self.offset, offset);
        let resolved = self.resolve_ty(ty_ast);
        self.cur_mod = sm;
        self.offset = so;
        let Ty::Tuple(items) = resolved else {
            return None;
        };
        let factors: Vec<_> = items
            .iter()
            .map(|t| self.ty_to_product_factor(t).map(|f| (None, f)))
            .collect::<Option<_>>()?;
        (factors.len() > 1).then_some(factors)
    }

    /// A walk over every position of the product of `factors`.
    pub(super) fn product_walk(
        &mut self,
        factors: Vec<(Option<String>, ProductFactor)>,
        span: Span,
    ) -> (Iter, Ty) {
        let d = self.intern_anon_product_domain(factors);
        let mut defs = Vec::new();
        self.def_anon_product_domain(d, span, &mut defs);
        self.cx.pending.extend(defs);
        let lo = self.int_lit(0, span);
        let hi = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
        let walk = Iter::Range {
            lo,
            hi,
            step: None,
            rev: false,
            closed: false,
        };
        (walk, Ty::Pos(d))
    }

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

    /// `read()` of a product position: each part in order, then the
    /// position they name.
    pub(super) fn product_read(&mut self, id: DomainId) -> Expr {
        let span = Span::default();
        let tys: Vec<Ty> = self.product_domains[&id]
            .iter()
            .map(|(_, f)| f.ty())
            .collect();
        let parts = Spec::Fields(
            Ty::Tuple(tys.clone()),
            tys.iter().cloned().map(Spec::Plain).collect(),
        );
        let parts = self.readable(parts);
        let read = self.mk(
            ExprKind::Read(Box::new(parts)),
            Ty::Tuple(tys.clone()),
            span,
        );
        let mut out = Vec::new();
        let held = self.hold(read, &mut out);
        let parts = (0..tys.len())
            .map(|i| {
                self.mk(
                    ExprKind::Field(Box::new(held.clone()), i),
                    tys[i].clone(),
                    span,
                )
            })
            .collect();
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

    pub(super) fn product_factor_ty(&self, f: &ProductFactor) -> Ty {
        f.ty()
    }

    pub(super) fn product_factor_len(&self, f: &ProductFactor, span: Span) -> Expr {
        match f {
            ProductFactor::Domain(d) => self.mk(ExprKind::DomainLen(*d), Ty::Int, span),
            ProductFactor::Enum { len, .. } => self.int_lit(*len, span),
            ProductFactor::Bool => self.int_lit(2, span),
            ProductFactor::Library { dom, .. } => self.mk(ExprKind::DomainLen(*dom), Ty::Int, span),
        }
    }

    pub(super) fn product_factor_ord(&mut self, f: &ProductFactor, val: Expr, span: Span) -> Expr {
        match f {
            ProductFactor::Domain(d) => {
                let v = self.coerce(val, &Ty::Pos(*d), span);
                self.cast(Cast::Ord, v, Ty::Int)
            }
            ProductFactor::Enum { enum_id, .. } => {
                let v = self.coerce(val, &Ty::Enum(*enum_id), span);
                self.cast(Cast::EnumOrd(*enum_id), v, Ty::Int)
            }
            ProductFactor::Bool => {
                let v = self.coerce(val, &Ty::Bool, span);
                let one = self.int_lit(1, span);
                let zero = self.int_lit(0, span);
                self.mk(
                    ExprKind::If(Box::new(v), Box::new(one), Box::new(zero)),
                    Ty::Int,
                    span,
                )
            }
            ProductFactor::Library { sid, ord, .. } => {
                let v = self.coerce(val, &Ty::Struct(*sid), span);
                self.mk(ExprKind::Call(*ord, vec![Arg::Value(v)]), Ty::Int, span)
            }
        }
    }

    pub(super) fn product_factor_from_ord(&self, f: &ProductFactor, ord: Expr, span: Span) -> Expr {
        match f {
            ProductFactor::Domain(d) => self.cast(Cast::Trust, ord, Ty::Pos(*d)),
            ProductFactor::Enum { enum_id, .. } => {
                self.cast(Cast::OrdToEnum(*enum_id), ord, Ty::Enum(*enum_id))
            }
            ProductFactor::Bool => {
                let zero = self.int_lit(0, span);
                self.mk(
                    ExprKind::Cmp(CmpOp::Ne, Box::new(ord), Box::new(zero)),
                    Ty::Bool,
                    span,
                )
            }
            ProductFactor::Library { sid, nth, .. } => self.mk(
                ExprKind::Call(*nth, vec![Arg::Value(ord)]),
                Ty::Struct(*sid),
                span,
            ),
        }
    }
}
