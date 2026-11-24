//! Binds a pattern's names to a value: a tuple of parts binds each name to
//! its part, so a read lends in place.

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn bind_mut(
        &mut self,
        pat: &ast::Pattern,
        value: Expr,
        mutable: bool,
        out: &mut Vec<Stmt>,
    ) {
        match &pat.kind {
            ast::PatternKind::Name(name) => self.declare(name, value, mutable, out),
            ast::PatternKind::Domain(name) => {
                let dname = &name.text;
                let dom_id = if self.at_top_level() {
                    if let Some(Sym::Domain(existing)) =
                        self.mod_syms[self.cur_mod].get(dname).copied()
                    {
                        existing
                    } else {
                        let id = self.domains.len();
                        self.domains.push(format!("{dname}_{id}"));
                        self.mod_syms[self.cur_mod].insert(dname.clone(), Sym::Domain(id));
                        id
                    }
                } else {
                    let id = self.domains.len();
                    self.domains.push(format!("{dname}_{id}"));
                    self.cx
                        .local_syms
                        .last_mut()
                        .unwrap()
                        .insert(dname.clone(), Sym::Domain(id));
                    id
                };
                out.push(Stmt::DefDomainFromVal {
                    id: dom_id,
                    val: value,
                });
            }
            ast::PatternKind::Variant { path, args } if path.len() == 1 && args.is_empty() => {
                // Uppercase variable binding like `let P = p` or `let V = g.V` (parsed as Variant).
                let vname = &path[0].text;
                if value.ty == Ty::DomainVal {
                    if let ExprKind::DomainVal(dom_id) = value.kind {
                        self.cx
                            .local_syms
                            .last_mut()
                            .unwrap()
                            .insert(vname.clone(), Sym::Domain(dom_id));
                        return;
                    }
                    let dom_id = self.domains.len();
                    self.domains.push(format!("{vname}_{dom_id}"));
                    out.push(Stmt::DefDomainFromVal {
                        id: dom_id,
                        val: value,
                    });
                    self.cx
                        .local_syms
                        .last_mut()
                        .unwrap()
                        .insert(vname.clone(), Sym::Domain(dom_id));
                    return;
                }
                self.declare(vname, value, mutable, out);
            }
            ast::PatternKind::Wildcard => out.push(Stmt::Expr(value)),
            ast::PatternKind::Record { .. } => {
                if let Some(tuple) = self.record_pattern(pat, &value.ty) {
                    self.bind_mut(&tuple, value, mutable, out);
                }
            }
            ast::PatternKind::Tuple(pats) => {
                if let Ty::Pos(dom_id) = value.ty {
                    if let Some(factors) = self.product_domains.get(&dom_id).cloned() {
                        if factors.len() != pats.len() {
                            let message = format!(
                                "this pattern has {} names, but `{}` has {} factors",
                                pats.len(),
                                self.show(&value.ty),
                                factors.len()
                            );
                            self.error(codes::TYPE_MISMATCH, pat.span, message);
                            return;
                        }
                        let parts = self.decode_product(dom_id, value, pat.span, out);
                        for (p, part) in pats.iter().zip(parts) {
                            self.bind_mut(p, part, mutable, out);
                        }
                        return;
                    }
                }
                let Some(fields) = value.ty.fields() else {
                    if value.ty != Ty::Error {
                        let message = format!(
                            "a `{}` can't be taken apart into {} names",
                            self.show(&value.ty),
                            pats.len()
                        );
                        self.error(codes::TYPE_MISMATCH, pat.span, message);
                    }
                    return;
                };
                if fields.len() != pats.len() {
                    let message = format!(
                        "this pattern has {} names, but the value has {} fields",
                        pats.len(),
                        fields.len()
                    );
                    self.error(codes::TYPE_MISMATCH, pat.span, message);
                    return;
                }
                // `let (a, b) = (xs[i], xs[j])`: a name per part reads in place, not
                // a tuple of copies; parts read old names first, so `(y, x)` swaps.
                let plain = |ty: &Ty| *ty != Ty::DomainVal && find_pos_domain_in_ty(ty).is_none();
                if let ExprKind::Tuple(parts) = &value.kind {
                    if parts.len() == pats.len()
                        && parts
                            .iter()
                            .zip(fields.iter())
                            .all(|(p, ty)| p.ty == *ty && plain(ty))
                    {
                        let ExprKind::Tuple(parts) = value.kind else {
                            return;
                        };
                        pats.iter()
                            .zip(parts)
                            .for_each(|(p, e)| self.bind_mut(p, e, mutable, out));
                        return;
                    }
                }
                // Identify the callee's existential domain id(s) if any field is `Ty::DomainVal`.
                let callee_doms: Vec<DomainId> = match &given(&value).kind {
                    ExprKind::Call(fn_id, _) => self.sigs[*fn_id]
                        .new_domain_slots
                        .iter()
                        .map(|(_, d)| *d)
                        .collect(),
                    ExprKind::Tuple(items) => items
                        .iter()
                        .filter_map(|e| match e.kind {
                            ExprKind::DomainVal(d) => Some(d),
                            _ => None,
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                let fallback_dom = fields.iter().find_map(find_pos_domain_in_ty);
                let mut remaps: Vec<(DomainId, DomainId)> = Vec::new();
                let mut dom_idx = 0usize;
                let base = self.hold(value, out);
                for (i, (p, ty)) in pats.iter().zip(fields).enumerate() {
                    let mut field = Expr {
                        kind: ExprKind::Field(Box::new(base.clone()), i),
                        ty: ty.clone(),
                        span: base.span,
                    };
                    for &(from, to) in &remaps {
                        field = self.remap_domain_in_expr(field, from, to, out);
                    }
                    if ty == Ty::DomainVal {
                        let dname_opt = match &p.kind {
                            ast::PatternKind::Domain(n) => Some(n.text.clone()),
                            ast::PatternKind::Name(n) => Some(n.clone()),
                            ast::PatternKind::Variant { path, args }
                                if args.is_empty() && path.len() == 1 =>
                            {
                                Some(path[0].text.clone())
                            }
                            _ => None,
                        };
                        if let Some(dname) = dname_opt {
                            // Reuse the pre-registered top-level domain slot when present so
                            // sibling top-level `fn` signatures refer to the same `DomainId`.
                            // Otherwise allocate a fresh runtime slot per binding site so
                            // multiple calls to the same existential-returning fn never alias.
                            let fresh_id = if self.at_top_level() {
                                if let Some(Sym::Domain(existing)) =
                                    self.mod_syms[self.cur_mod].get(&dname).copied()
                                {
                                    existing
                                } else {
                                    let id = self.domains.len();
                                    self.domains.push(format!("{dname}_{id}"));
                                    self.mod_syms[self.cur_mod]
                                        .insert(dname.clone(), Sym::Domain(id));
                                    id
                                }
                            } else {
                                let id = self.domains.len();
                                self.domains.push(format!("{dname}_{id}"));
                                self.cx
                                    .local_syms
                                    .last_mut()
                                    .unwrap()
                                    .insert(dname.clone(), Sym::Domain(id));
                                id
                            };
                            if let Some(orig) = callee_doms.get(dom_idx).copied().or(fallback_dom) {
                                remaps.push((orig, fresh_id));
                            }
                            dom_idx += 1;
                            out.push(Stmt::DefDomainFromVal {
                                id: fresh_id,
                                val: field,
                            });
                            continue;
                        }
                    }
                    self.bind_mut(p, field, mutable, out);
                }
            }
            ast::PatternKind::Var(inner) => self.bind_mut(inner, value, true, out),
            ast::PatternKind::Error => {}
            _ => self.unsupported(pat.span, "this pattern outside a `match`"),
        }
    }
}
