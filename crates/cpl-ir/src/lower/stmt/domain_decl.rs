//! `domain` declarations: sums of domains, runtime domain ids, and remapping
//! one domain to another in expressions.

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn domain(
        &mut self,
        name: &ast::Name,
        def: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) {
        let id = if self.at_top_level() {
            match self.mod_syms[self.cur_mod].get(&name.text).copied() {
                Some(Sym::Domain(id)) => id,
                _ => return,
            }
        } else if let Some(Sym::Domain(id)) = self
            .cx
            .local_syms
            .iter()
            .rev()
            .find_map(|s| s.get(&name.text).copied())
        {
            // Reuse pre-allocated domain id (e.g. `domain C = 0..k` inside `scc` returning `domain C`).
            id
        } else {
            let id = self.domains.len();
            self.domains.push(format!("{}_{id}", name.text));
            self.cx
                .local_syms
                .last_mut()
                .unwrap()
                .insert(name.text.clone(), Sym::Domain(id));
            id
        };
        self.init_domain_id(id, def, out);
    }

    /// `domain Node = In + Out`: the domains of a disjoint union, in order.
    pub(in crate::lower) fn domain_sum_parts(&mut self, def: &ast::Expr) -> Option<Vec<DomainId>> {
        let ast::ExprKind::Binary {
            op: ast::BinOp::Add,
            lhs,
            rhs,
            ..
        } = &def.kind
        else {
            return None;
        };
        let mut parts = self.domain_sum_part(lhs)?;
        parts.extend(self.domain_sum_part(rhs)?);
        Some(parts)
    }

    pub(in crate::lower) fn domain_sum_part(&mut self, e: &ast::Expr) -> Option<Vec<DomainId>> {
        match &e.kind {
            ast::ExprKind::Name(name) => self.resolve_domain_name(name).map(|d| vec![d]),
            _ => self.domain_sum_parts(e),
        }
    }

    pub(in crate::lower) fn init_domain_id(
        &mut self,
        id: DomainId,
        def: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) {
        // `domain E = [m]` is parsed as `0..m`; the brackets mark it counted.
        if self.source(def.span).trim_start().starts_with('[') {
            self.counted.insert(id);
            self.counted_domains.insert(id);
        }
        if let ast::ExprKind::List(items) = &def.kind {
            if items.len() == 1 {
                let val = self.expr(&items[0], Some(&Ty::Int));
                let len = self.coerce(val, &Ty::Int, def.span);
                let lo = self.mk(ExprKind::Int(0), Ty::Int, def.span);
                self.counted.insert(id);
                self.counted_domains.insert(id);
                out.push(Stmt::DefDomain { id, lo, len });
                return;
            }
        }
        if let Some(factors) = self.resolve_product_factors(def) {
            self.def_product_factors(id, def, &factors, out);
            let mut len = self.product_factor_len(&factors[0].1, def.span);
            for (_, f) in &factors[1..] {
                let flen = self.product_factor_len(f, def.span);
                len = self.mk_binary(BinOp::Mul, len, flen, Ty::Int, def.span);
            }
            let lo = self.int_lit(0, def.span);
            out.push(Stmt::DefDomain { id, lo, len });
            return;
        }
        if let Some(parts) = self.domain_sum_parts(def) {
            let mut lens = parts
                .into_iter()
                .map(|d| self.mk(ExprKind::DomainLen(d), Ty::Int, def.span));
            let first = lens.next().unwrap();
            let len = lens.fold(first, |acc, l| {
                self.mk_binary(BinOp::Add, acc, l, Ty::Int, def.span)
            });
            let lo = self.int_lit(0, def.span);
            out.push(Stmt::DefDomain { id, lo, len });
            return;
        }
        if let Some(base_id) = self.domain_pairs_iter(def) {
            let n = self.mk(ExprKind::DomainLen(base_id), Ty::Int, def.span);
            let n_minus_1 = self.int_add(n.clone(), -1);
            let prod = self.mk_binary(BinOp::Mul, n, n_minus_1, Ty::Int, def.span);
            let two = self.int_lit(2, def.span);
            let len = self.mk_binary(BinOp::Div, prod, two, Ty::Int, def.span);
            let lo = self.int_lit(0, def.span);
            out.push(Stmt::DefDomain { id, lo, len });
            return;
        }
        let ast::ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            step: None,
        } = &def.kind
        else {
            // `domain V = expr`: supports counts (`Int`), existing domains (`DomainVal`),
            // existential-domain tuples (`(domain C, ...)`), sequences (`List`/`Array`/`Str`),
            // and domain-owning structs (e.g. `Compressed<D, T>` from `compress(xs)`).
            let val = self.expr(def, Some(&Ty::Int));
            match &val.ty {
                Ty::Int => {
                    self.counted.insert(id);
                    let len = self.coerce(val, &Ty::Int, def.span);
                    let lo = self.mk(ExprKind::Int(0), Ty::Int, def.span);
                    self.counted_domains.insert(id);
                    out.push(Stmt::DefDomain { id, lo, len });
                }
                Ty::DomainVal => {
                    out.push(Stmt::DefDomainFromVal { id, val });
                }
                Ty::Range(inner) if **inner == Ty::Int => {
                    let (lo, len) = self.range_lo_len(val, def.span);
                    out.push(Stmt::DefDomain { id, lo, len });
                }
                Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::Array { .. }
                | Ty::Str
                | Ty::StrView => {
                    let len = self.mk_builtin(Builtin::Len, [val], Ty::Int, def.span);
                    let lo = self.mk(ExprKind::Int(0), Ty::Int, def.span);
                    out.push(Stmt::DefDomain { id, lo, len });
                }
                Ty::Tuple(fields) if fields.first() == Some(&Ty::DomainVal) => {
                    let dom_field = self.mk_field(val, 0, Ty::DomainVal, def.span);
                    out.push(Stmt::DefDomainFromVal { id, val: dom_field });
                }
                Ty::Record(fields) if fields.first().is_some_and(|(_, t)| *t == Ty::DomainVal) => {
                    let dom_field = self.mk_field(val, 0, Ty::DomainVal, def.span);
                    out.push(Stmt::DefDomainFromVal { id, val: dom_field });
                }
                Ty::Struct(sid) => {
                    let sid = *sid;
                    // Check if the struct has an array/list/str field (e.g. `Compressed.vals`).
                    let len_expr = self.structs[sid]
                        .def
                        .fields
                        .iter()
                        .enumerate()
                        .find(|(_, (_, t))| matches!(t, Ty::Array { .. } | Ty::List(_) | Ty::Str))
                        .map(|(idx, (_, fty))| {
                            let fty = fty.clone();
                            let field = self.mk_field(val, idx, fty, def.span);
                            self.mk_builtin(Builtin::Len, [field], Ty::Int, def.span)
                        });
                    if let Some(len) = len_expr {
                        let lo = self.mk(ExprKind::Int(0), Ty::Int, def.span);
                        out.push(Stmt::DefDomain { id, lo, len });
                    } else {
                        self.unsupported(
                            def.span,
                            "a domain expression that does not produce a range, count, or index space",
                        );
                    }
                }
                Ty::Error => {}
                _ => {
                    self.unsupported(
                        def.span,
                        "a domain expression that does not produce a range, count, or index space",
                    );
                }
            }
            return;
        };
        if !*inclusive && matches!(lo.kind, ast::ExprKind::Int(0)) && lo.span.start == lo.span.end {
            self.counted_domains.insert(id);
        }
        if let (Some(l), Some(h)) = (self.ast_bounds(lo), self.ast_bounds(hi)) {
            let extra = i64::from(*inclusive);
            let least = h.0.checked_add(extra).and_then(|h| h.checked_sub(l.1));
            let most = h.1.checked_add(extra).and_then(|h| h.checked_sub(l.0));
            if let (Some(least), Some(most)) = (least, most) {
                self.domain_lens.insert(id, (least.max(0), most.max(0)));
            }
        }
        let lo = self.domain_end(id, lo);
        let hi = self.domain_end(id, hi);
        let slot = self.temp(Ty::Int);
        out.push(Stmt::Let(slot, Some(lo)));
        let lo = self.local(slot, def.span);
        let len = self.mk_binary(BinOp::Sub, hi, lo.clone(), Ty::Int, def.span);
        let len = if *inclusive {
            self.int_add(len, 1)
        } else {
            len
        };
        out.push(Stmt::DefDomain { id, lo, len });
    }

    pub(in crate::lower) fn remap_domain_in_expr(
        &mut self,
        expr: Expr,
        from: DomainId,
        to: DomainId,
        out: &mut Vec<Stmt>,
    ) -> Expr {
        if from == to {
            return expr;
        }
        if let Ty::Struct(old_sid) = expr.ty {
            if let Some((gid, old_args)) = self.structs[old_sid].origin_gen.clone() {
                let mut changed = false;
                let mut new_args = Vec::with_capacity(old_args.len());
                for ga in &old_args {
                    let mapped = match ga {
                        GenArg::Domain(d) if *d == from => {
                            changed = true;
                            GenArg::Domain(to)
                        }
                        GenArg::Ty(t) => {
                            let nt = remap_domain_ty(t, from, to);
                            if nt != *t {
                                changed = true;
                            }
                            GenArg::Ty(nt)
                        }
                        other => other.clone(),
                    };
                    new_args.push(mapped);
                }
                if changed {
                    let new_sid = self.instantiate_struct(gid, new_args);
                    let new_fields = self.structs[new_sid].def.fields.clone();
                    let old_fields = self.structs[old_sid].def.fields.clone();
                    let span = expr.span;
                    let held = self.hold(expr, out);
                    // By name: each instance keeps only the fields it uses, and a
                    // field's own type (a nested struct over `from`) needs the remap too.
                    let mut values = Vec::with_capacity(new_fields.len());
                    for (name, _) in &new_fields {
                        let Some(fi) = old_fields.iter().position(|(n, _)| n == name) else {
                            continue;
                        };
                        let field = Expr {
                            kind: ExprKind::Field(Box::new(held.clone()), fi),
                            ty: old_fields[fi].1.clone(),
                            span,
                        };
                        values.push(self.remap_domain_in_expr(field, from, to, out));
                    }
                    return Expr {
                        kind: ExprKind::Tuple(values),
                        ty: Ty::Struct(new_sid),
                        span,
                    };
                }
            }
        }
        let new_ty = remap_domain_ty(&expr.ty, from, to);
        Expr { ty: new_ty, ..expr }
    }
}
