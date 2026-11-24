//! Sum and product domains: their factors, anonymous product domains, and
//! encoding and decoding a position as its variant or its parts.

use super::*;

impl<'a> Lowerer<'a> {
    pub(crate) fn resolve_product_factors(
        &mut self,
        def: &ast::Expr,
    ) -> Option<Vec<(Option<String>, ProductFactor)>> {
        match &def.kind {
            ast::ExprKind::Tuple(items) if items.len() >= 2 => items
                .iter()
                .map(|item| Some((None, self.resolve_product_factor(item)?)))
                .collect(),
            ast::ExprKind::Record(fields) if fields.len() >= 2 => fields
                .iter()
                .map(|(n, v)| Some((Some(n.text.clone()), self.resolve_product_factor(v)?)))
                .collect(),
            _ => None,
        }
    }

    pub(crate) fn ty_to_product_factor(&self, ty: &Ty) -> Option<ProductFactor> {
        match ty {
            Ty::Pos(d) => Some(ProductFactor::Domain(*d)),
            Ty::Bool => Some(ProductFactor::Bool),
            Ty::Struct(sid) => {
                let s = &self.structs[*sid];
                Some(ProductFactor::Library {
                    sid: *sid,
                    dom: s.index_domain?,
                    ord: *s.methods.get("ord")?,
                    nth: *s.methods.get("nth")?,
                })
            }
            Ty::Enum(eid)
                if !self.enums[*eid].variants.is_empty()
                    && self.enums[*eid]
                        .variants
                        .iter()
                        .all(|(_, fs)| fs.is_empty()) =>
            {
                Some(ProductFactor::Enum {
                    enum_id: *eid,
                    len: self.enums[*eid].variants.len() as i64,
                })
            }
            _ => None,
        }
    }

    pub(crate) fn expr_to_product_factor(&self, e: &ast::Expr) -> Option<ProductFactor> {
        match &e.kind {
            ast::ExprKind::Paren(inner) => self.expr_to_product_factor(inner),
            ast::ExprKind::Name(name) if name == "Bool" && self.lookup(name).is_none() => {
                Some(ProductFactor::Bool)
            }
            _ => match self.sym(e) {
                Some(Sym::Domain(id)) => Some(ProductFactor::Domain(id)),
                Some(Sym::Enum(eid)) => {
                    if let Some(&sum_dom) = self.enum_to_sum_domain.get(&eid) {
                        Some(ProductFactor::Domain(sum_dom))
                    } else if self.enums[eid].variants.iter().all(|(_, fs)| fs.is_empty()) {
                        Some(ProductFactor::Enum {
                            enum_id: eid,
                            len: self.enums[eid].variants.len() as i64,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            },
        }
    }

    pub(crate) fn intern_anon_product_domain(
        &mut self,
        factors: Vec<(Option<String>, ProductFactor)>,
    ) -> DomainId {
        if let Some(&d) = self.anon_product_domains.get(&factors) {
            return d;
        }
        let d = self.domains.len();
        self.domains.push(format!("Prod{d}"));
        self.product_domains.insert(d, factors.clone());
        self.anon_product_domains.insert(factors, d);
        self.anon_product_domain_set.insert(d);
        d
    }

    pub(crate) fn def_anon_product_domain(&self, d: DomainId, span: Span, out: &mut Vec<Stmt>) {
        let Some(factors) = self.product_domains.get(&d) else {
            return;
        };
        let mut total = self.product_factor_len(&factors[0].1, span);
        for (_, f) in &factors[1..] {
            let flen = self.product_factor_len(f, span);
            total = self.mk(
                ExprKind::Binary(BinOp::Mul, Box::new(total), Box::new(flen)),
                Ty::Int,
                span,
            );
        }
        out.push(Stmt::DefDomain {
            id: d,
            lo: self.int_lit(0, span),
            len: total,
        });
    }

    pub(crate) fn sum_variant_count(&self, factors: &[ProductFactor], span: Span) -> Expr {
        if factors.is_empty() {
            return self.int_lit(1, span);
        }
        let mut prod = self.product_factor_len(&factors[0], span);
        for f in &factors[1..] {
            let flen = self.product_factor_len(f, span);
            prod = self.mk(
                ExprKind::Binary(BinOp::Mul, Box::new(prod), Box::new(flen)),
                Ty::Int,
                span,
            );
        }
        prod
    }

    pub(crate) fn sum_variant_offset(&self, d: DomainId, tag: usize, span: Span) -> Expr {
        let variants = &self.sum_enums[&d].variants;
        if tag == 0 {
            return self.int_lit(0, span);
        }
        let mut acc = self.sum_variant_count(&variants[0].1, span);
        for (_, fs) in &variants[1..tag] {
            let c = self.sum_variant_count(fs, span);
            acc = self.mk(
                ExprKind::Binary(BinOp::Add, Box::new(acc), Box::new(c)),
                Ty::Int,
                span,
            );
        }
        acc
    }

    pub(crate) fn def_sum_enum_domain(&self, d: DomainId, span: Span, out: &mut Vec<Stmt>) {
        let variants = &self.sum_enums[&d].variants;
        let mut total = self.sum_variant_count(&variants[0].1, span);
        for (_, fs) in &variants[1..] {
            let c = self.sum_variant_count(fs, span);
            total = self.mk(
                ExprKind::Binary(BinOp::Add, Box::new(total), Box::new(c)),
                Ty::Int,
                span,
            );
        }
        out.push(Stmt::DefDomain {
            id: d,
            lo: self.int_lit(0, span),
            len: total,
        });
    }

    pub(crate) fn encode_sum_variant(
        &mut self,
        d: DomainId,
        tag: usize,
        parts: Vec<Expr>,
        span: Span,
    ) -> Expr {
        let factors = self.sum_enums[&d].variants[tag].1.clone();
        let off = self.sum_variant_offset(d, tag, span);
        let ord = if factors.is_empty() {
            off
        } else {
            let mut inner = self.product_factor_ord(&factors[0], parts[0].clone(), span);
            for (f, part) in factors[1..].iter().zip(parts.into_iter().skip(1)) {
                let len = self.product_factor_len(f, span);
                let ford = self.product_factor_ord(f, part, span);
                let scaled = self.mk(
                    ExprKind::Binary(BinOp::Mul, Box::new(inner), Box::new(len)),
                    Ty::Int,
                    span,
                );
                inner = self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(scaled), Box::new(ford)),
                    Ty::Int,
                    span,
                );
            }
            if tag == 0 {
                inner
            } else {
                self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(off), Box::new(inner)),
                    Ty::Int,
                    span,
                )
            }
        };
        self.cast(Cast::Trust, ord, Ty::Pos(d))
    }

    pub(crate) fn is_sum_variant(&self, d: DomainId, tag: usize, val: Expr, span: Span) -> Expr {
        let factors = &self.sum_enums[&d].variants[tag].1;
        let ord = self.cast(Cast::Ord, val, Ty::Int);
        let lo = self.sum_variant_offset(d, tag, span);
        if factors.is_empty() {
            return self.mk(
                ExprKind::Cmp(CmpOp::Eq, Box::new(ord), Box::new(lo)),
                Ty::Bool,
                span,
            );
        }
        let count = self.sum_variant_count(factors, span);
        let hi = self.mk(
            ExprKind::Binary(BinOp::Add, Box::new(lo.clone()), Box::new(count)),
            Ty::Int,
            span,
        );
        let ge = self.mk(
            ExprKind::Cmp(CmpOp::Ge, Box::new(ord.clone()), Box::new(lo)),
            Ty::Bool,
            span,
        );
        let lt = self.mk(
            ExprKind::Cmp(CmpOp::Lt, Box::new(ord), Box::new(hi)),
            Ty::Bool,
            span,
        );
        self.mk(
            ExprKind::Binary(BinOp::And, Box::new(ge), Box::new(lt)),
            Ty::Bool,
            span,
        )
    }

    pub(crate) fn decode_sum_variant_field(
        &self,
        d: DomainId,
        tag: usize,
        field_idx: usize,
        val: Expr,
        span: Span,
    ) -> Expr {
        let factors = &self.sum_enums[&d].variants[tag].1;
        let ord = self.cast(Cast::Ord, val, Ty::Int);
        let mut rel = if tag == 0 {
            ord
        } else {
            let lo = self.sum_variant_offset(d, tag, span);
            self.mk(
                ExprKind::Binary(BinOp::Sub, Box::new(ord), Box::new(lo)),
                Ty::Int,
                span,
            )
        };
        if field_idx + 1 < factors.len() {
            let mut stride = self.product_factor_len(&factors[field_idx + 1], span);
            for f in &factors[field_idx + 2..] {
                let flen = self.product_factor_len(f, span);
                stride = self.mk(
                    ExprKind::Binary(BinOp::Mul, Box::new(stride), Box::new(flen)),
                    Ty::Int,
                    span,
                );
            }
            rel = self.mk(
                ExprKind::Binary(BinOp::Div, Box::new(rel), Box::new(stride)),
                Ty::Int,
                span,
            );
        }
        let ford = if field_idx == 0 {
            rel
        } else {
            let flen = self.product_factor_len(&factors[field_idx], span);
            self.mk(
                ExprKind::Binary(BinOp::Rem, Box::new(rel), Box::new(flen)),
                Ty::Int,
                span,
            )
        };
        self.product_factor_from_ord(&factors[field_idx], ford, span)
    }

    pub(crate) fn encode_product(&mut self, id: DomainId, parts: Vec<Expr>, span: Span) -> Expr {
        let factors = self.product_domains[&id].clone();
        let mut acc = self.product_factor_ord(&factors[0].1, parts[0].clone(), span);
        for ((_, f), part) in factors[1..].iter().zip(parts.into_iter().skip(1)) {
            let len = self.product_factor_len(f, span);
            let ord = self.product_factor_ord(f, part, span);
            let scaled = self.mk(
                ExprKind::Binary(BinOp::Mul, Box::new(acc), Box::new(len)),
                Ty::Int,
                span,
            );
            acc = self.mk(
                ExprKind::Binary(BinOp::Add, Box::new(scaled), Box::new(ord)),
                Ty::Int,
                span,
            );
        }
        self.cast(Cast::Trust, acc, Ty::Pos(id))
    }

    pub(crate) fn decode_product(
        &mut self,
        id: DomainId,
        val: Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) -> Vec<Expr> {
        if let Some(parts) = self.loop_parts_of(&val) {
            return parts;
        }
        let factors = self.product_domains[&id].clone();
        let held = self.hold(val, out);
        let mut ord = self.cast(Cast::Ord, held, Ty::Int);
        let mut parts = vec![self.int_lit(0, span); factors.len()];
        for i in (1..factors.len()).rev() {
            let ord_held = self.hold(ord, out);
            let len = self.product_factor_len(&factors[i].1, span);
            let rem = self.mk(
                ExprKind::Binary(
                    BinOp::Rem,
                    Box::new(ord_held.clone()),
                    Box::new(len.clone()),
                ),
                Ty::Int,
                span,
            );
            parts[i] = self.product_factor_from_ord(&factors[i].1, rem, span);
            ord = self.mk(
                ExprKind::Binary(BinOp::Div, Box::new(ord_held), Box::new(len)),
                Ty::Int,
                span,
            );
        }
        parts[0] = self.product_factor_from_ord(&factors[0].1, ord, span);
        parts
    }

    /// The grid dimensions `[A, B]` a position of the product domain `id`
    /// indexes whole: `[P]T` and `[A, B]T` are one type (R04.2.11).
    pub(crate) fn product_grid(&self, id: DomainId) -> Option<Vec<Dim>> {
        let factors = self.product_domains.get(&id).filter(|fs| fs.len() > 1)?;
        factors
            .iter()
            .map(|(_, f)| match f {
                ProductFactor::Domain(d) => Some(Dim::Pos(*d)),
                _ => None,
            })
            .collect()
    }

    /// The product domain whose grid is `dims`, a declared one before an anonymous one.
    pub(crate) fn product_of_grid(&self, dims: &[Dim]) -> Option<DomainId> {
        (self.product_domains.keys().copied())
            .filter(|&id| self.product_grid(id).as_deref() == Some(dims))
            .min_by_key(|id| (self.anon_product_domain_set.contains(id), *id))
    }

    /// Registers the product domain each `V` of an `[V, V]T` parameter is when given an
    /// `[A, B, A, B]T`.
    pub(crate) fn intern_grid_products(&mut self, pat: &ast::Type, concrete: &Ty) {
        let (ast::TypeKind::Array { index, .. }, Ty::Array { dims, .. }) = (&pat.kind, concrete)
        else {
            return;
        };
        let per = dims.len() / index.len().max(1);
        if per < 2 || dims.len() != per * index.len() {
            return;
        }
        for chunk in dims.chunks(per) {
            let factors: Option<Vec<_>> = chunk
                .iter()
                .map(|d| match d {
                    Dim::Pos(d) => Some((None, ProductFactor::Domain(*d))),
                    _ => None,
                })
                .collect();
            let Some(factors) = factors else { continue };
            let id = match self.product_of_grid(chunk) {
                Some(id) => id,
                None => self.intern_anon_product_domain(factors),
            };
            if self.anon_product_domain_set.contains(&id) {
                let mut defs = Vec::new();
                self.def_anon_product_domain(id, pat.span, &mut defs);
                self.cx.pending.extend(defs);
            }
        }
    }

    pub(crate) fn decode_product_field(
        &mut self,
        id: DomainId,
        val: Expr,
        idx: usize,
        span: Span,
    ) -> Expr {
        if let Some(mut parts) = self.loop_parts_of(&val) {
            return parts.swap_remove(idx);
        }
        let factors = self.product_domains[&id].clone();
        let mut ord = self.cast(Cast::Ord, val, Ty::Int);
        for (_, f) in &factors[idx + 1..] {
            let len = self.product_factor_len(f, span);
            ord = self.mk(
                ExprKind::Binary(BinOp::Div, Box::new(ord), Box::new(len)),
                Ty::Int,
                span,
            );
        }
        if idx > 0 {
            let len = self.product_factor_len(&factors[idx].1, span);
            ord = self.mk(
                ExprKind::Binary(BinOp::Rem, Box::new(ord), Box::new(len)),
                Ty::Int,
                span,
            );
        }
        self.product_factor_from_ord(&factors[idx].1, ord, span)
    }

    /// The factors of a position a product loop binds, which its loops walk.
    fn loop_parts_of(&self, val: &Expr) -> Option<Vec<Expr>> {
        match &val.kind {
            ExprKind::Local(slot) => self.cx.product_parts.get(slot).cloned(),
            _ => None,
        }
    }
}
