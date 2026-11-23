//! Generators: `yield` pushes onto the hidden list a `-> yields T` fn builds,
//! and a generic only `-> yields G` names takes the type its body yields:
//! `present<S: Seq, T>(self: S) -> yields T` over `?Range<Int>` pieces yields
//! `Range<Int>`, since its body yields the unwrapped `y`.

use super::*;

impl<'a> Lowerer<'a> {
    /// `yield v` pushes `v` onto the generator's list; `yield from s` extends it by `s`.
    pub(super) fn yield_stmt(
        &mut self,
        value: &ast::Expr,
        from: bool,
        span: Span,
        out: &mut Vec<Stmt>,
    ) {
        if let Some(mut seen) = self.cx.yield_seen.take() {
            let v = self.expr(value, None);
            seen.push(if from {
                v.ty.seq_elem().unwrap_or(Ty::Error)
            } else {
                v.ty
            });
            self.cx.yield_seen = Some(seen);
            return;
        }
        let Some((acc, elem_ty)) = self.cx.yield_slot.clone() else {
            self.error(codes::UNSUPPORTED, span, "`yield` outside a generator");
            return;
        };
        let acc_place = Place {
            kind: PlaceKind::Local(acc),
            ty: Ty::list(elem_ty.clone()),
            span: self.span(span),
        };
        if from {
            let seq = self.expr(value, None);
            let push_call = self.mk(
                ExprKind::Mutate(Mutation::Extend, Box::new(acc_place), vec![seq]),
                Ty::Unit,
                span,
            );
            out.push(Stmt::Expr(push_call));
        } else {
            let v = self.expect(value, &elem_ty);
            let push_call = self.mk(
                ExprKind::Mutate(Mutation::Push, Box::new(acc_place), vec![v]),
                Ty::Unit,
                span,
            );
            out.push(Stmt::Expr(push_call));
        }
    }

    /// Bind `G` in `-> yields G` to what the block body yields once every parameter's type is known.
    pub(super) fn infer_yielded(
        &mut self,
        decl: &'a ast::FnDecl,
        (tpl_mod, tpl_offset, free): (ModId, u32, bool),
        gen_names: &[String],
        inferred: &mut GenEnv,
    ) {
        let Some(ast::TypeKind::Yields(elem)) = decl.ret.as_ref().map(|t| &t.kind) else {
            return;
        };
        let ast::TypeKind::Named { name, args } = &elem.kind else {
            return;
        };
        let ast::FnBody::Block(block) = &decl.body else {
            return;
        };
        let mut env = inferred.clone();
        env.remove(&name.text);
        if !free
            || !args.is_empty()
            || !gen_names.contains(&name.text)
            || matches!(inferred.get(&name.text), Some(ga) if !matches!(ga, GenArg::Ty(_)))
            || anon_domain::fn_uses_domain(decl, &name.text)
            || decl.params.iter().any(|p| {
                p.ty.as_ref().is_none_or(|t| {
                    gen_names
                        .iter()
                        .any(|g| ty_mentions(t, g) && (g == &name.text || !env.contains_key(g)))
                })
            })
        {
            return;
        }
        let diag_len = self.diags.len();
        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
        let saved_env = std::mem::replace(&mut self.gen_env, env);
        let saved_outer = std::mem::take(&mut self.outer_cxs);
        let fresh = FnCx {
            scopes: vec![HashMap::new()],
            local_syms: vec![HashMap::new()],
            ..FnCx::default()
        };
        let saved_cx = std::mem::replace(&mut self.cx, fresh);
        for p in &decl.params {
            let ty = match p.ty.as_ref().map(|t| self.resolve_ty(t)) {
                Some(Ty::Mut(inner)) => *inner,
                Some(ty) => ty,
                None => Ty::Error,
            };
            let slot = self.new_slot(&p.name.text, ty);
            self.cx.scopes[0].insert(p.name.text.clone(), slot);
            if p.mutable {
                self.cx.mut_slots.insert(slot);
            }
        }
        let acc = self.temp(Ty::list(Ty::Error));
        self.cx.yield_slot = Some((acc, Ty::Error));
        self.cx.yield_seen = Some(Vec::new());
        self.block(block, Tail::Discard);
        let seen = self.cx.yield_seen.take().unwrap_or_default();
        self.cx = saved_cx;
        self.outer_cxs = saved_outer;
        self.gen_env = saved_env;
        self.offset = saved_offset;
        self.cur_mod = saved_mod;
        self.diags.truncate(diag_len);
        if let Some(first) = seen.first() {
            if *first != Ty::Error && seen.iter().all(|t| t == first) {
                inferred.insert(name.text.clone(), GenArg::Ty(first.clone()));
            }
        }
    }
}
