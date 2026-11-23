//! Places a `lends mut` call names: `head(mut a) = 7`, `a.at(i) = v`.

use super::*;

impl<'a> Lowerer<'a> {
    /// `a.at(i)` on a list, array or view is the place `a[i]`: `at` is
    /// the built-in `lends mut` element (R05.6.3).
    pub(super) fn builtin_at(&mut self, e: &ast::Expr) -> Option<ast::Expr> {
        let ast::ExprKind::Call { callee, args } = &e.kind else {
            return None;
        };
        let (ast::ExprKind::Field { base, name }, [arg]) = (&callee.kind, args.as_slice()) else {
            return None;
        };
        if name.text != "at" || arg.name.is_some() || arg.mutable {
            return None;
        }
        if !self.probe_ty_silent(base)?.is_indexed() {
            return None;
        }
        Some(ast::Expr {
            kind: ast::ExprKind::Index {
                base: base.clone(),
                indices: vec![arg.value.clone()],
            },
            span: e.span,
        })
    }

    /// `g[u]` on a struct with an `at` method and no `get` is `g.at(u)` (R05.6.3). When `at`
    /// takes a cut of `D`, a position `v` of `D` is its right cut `v.r` (R07.1.6).
    pub(in crate::lower) fn struct_at(
        &mut self,
        sid: StructId,
        base: &ast::Expr,
        indices: &[ast::Expr],
    ) -> Option<ast::Expr> {
        let [index] = indices else {
            return None;
        };
        if matches!(index.kind, ast::ExprKind::Range { .. }) {
            return None;
        }
        let info = &self.structs[sid];
        let has_at = info.methods.contains_key("at")
            || info.gen_methods.contains_key("at")
            || info.projections.contains_key("at");
        if self.counter_key.contains_key(&sid) || !has_at {
            return None;
        }
        let span = Span::new(base.span.start, index.span.end + 1);
        let at = self.structs[sid].methods.get("at").copied();
        if let Some(f) = at {
            if !self.ensure_sig(f, Some(span)) {
                return None;
            }
        }
        let index = match at.map(|f| &self.sigs[f].params) {
            Some(params) if matches!(params.last().map(|p| &p.ty), Some(Ty::Bound(_))) => {
                let Some(Ty::Bound(d)) = params.last().map(|p| p.ty.clone()) else {
                    unreachable!()
                };
                if self.probe_ty_silent(index) == Some(Ty::Pos(d)) {
                    let name = ast::Name {
                        text: "r".into(),
                        span: index.span,
                    };
                    &ast::Expr {
                        kind: ast::ExprKind::Field {
                            base: Box::new(index.clone()),
                            name,
                        },
                        span: index.span,
                    }
                } else {
                    index
                }
            }
            _ => index,
        };
        let name = ast::Name {
            text: "at".into(),
            span: index.span,
        };
        let callee = ast::Expr {
            kind: ast::ExprKind::Field {
                base: Box::new(base.clone()),
                name,
            },
            span,
        };
        let arg = ast::Arg {
            name: None,
            mutable: false,
            value: index.clone(),
        };
        Some(ast::Expr {
            kind: ast::ExprKind::Call {
                callee: Box::new(callee),
                args: vec![arg],
            },
            span,
        })
    }

    pub(super) fn free_lends_mut_fn(&self, name: &str) -> Option<(u32, &'a ast::FnDecl)> {
        self.mod_items(self.cur_mod)
            .find_map(|(offset, stmt)| match &stmt.kind {
                ast::StmtKind::Fn(d) if d.name.text == name && d.lends.is_some() => {
                    Some((offset, d))
                }
                _ => None,
            })
    }

    /// `head(mut a) = 7` for `fn head(mut xs: List<Int>) lends mut Int = xs[0]`:
    /// each `mut` parameter names the caller's place, the others are bound
    /// once, and the lent place is assigned.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn lower_free_lend_assign(
        &mut self,
        decl: &'a ast::FnDecl,
        offset: u32,
        args: &[ast::Arg],
        op: ast::AssignOp,
        rhs: &ast::Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) {
        let (pre, lent, post) = super::mono::split_projection_body(decl);
        let Some(lent) = lent else {
            self.error(
                codes::UNSUPPORTED,
                span,
                "projection has no `lend` statement",
            );
            return;
        };
        let rhs_val = self.expr(rhs, None);
        let rhs_slot = self.temp(rhs_val.ty.clone());
        out.push(Stmt::Let(rhs_slot, Some(rhs_val)));
        let saved_aliases = self.cx.mut_aliases.clone();
        let mut bound = Vec::new();
        for (p, arg) in decl.params.iter().zip(args) {
            self.cx.mut_aliases.remove(&p.name.text);
            if p.mutable {
                let inner = match &arg.value.kind {
                    ast::ExprKind::Mut(inner) => inner,
                    _ => &arg.value,
                };
                let place = self.place(inner);
                let place = self.freeze_place(place, out);
                self.cx.mut_aliases.insert(p.name.text.clone(), place);
            } else {
                let saved = std::mem::replace(&mut self.offset, offset);
                let ty = p.ty.as_ref().map(|t| self.resolve_ty(t)).unwrap_or(Ty::Int);
                self.offset = saved;
                let v = self.expect(&arg.value, &ty);
                let slot = self.new_slot(&p.name.text, ty);
                out.push(Stmt::Let(slot, Some(v)));
                bound.push((p.name.text.clone(), slot));
            }
        }
        let saved_offset = std::mem::replace(&mut self.offset, offset);
        self.cx.scopes.push(bound.into_iter().collect());
        for s in pre {
            self.stmt(s, out);
        }
        let target = self.place(lent);
        let val = self.local(rhs_slot, rhs.span);
        let val = self.coerce(val, &target.ty, rhs.span);
        self.emit_assign_place(target, op, val, span, out);
        for s in post {
            self.stmt(s, out);
        }
        self.cx.scopes.pop();
        self.offset = saved_offset;
        self.cx.mut_aliases = saved_aliases;
    }
}
