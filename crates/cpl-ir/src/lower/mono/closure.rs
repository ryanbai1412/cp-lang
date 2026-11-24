//! Local fns, lambdas, and fns or bound methods as closure values.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// A fn written inside another, lowered as a closure bound to its name. `into` is the table of a
    /// `@memo(into: dp)`, which the caller has declared.
    pub(in crate::lower) fn lower_local_fn(
        &mut self,
        decl: &ast::FnDecl,
        memo: bool,
        into: Option<&str>,
        out: &mut Vec<Stmt>,
    ) {
        if !decl.generics.is_empty() {
            self.cx
                .local_gen_fns
                .insert(decl.name.text.clone(), decl.clone());
            return;
        }
        if Self::liftable_gen(decl)
            && !memo
            && into.is_none()
            && !self.cx.predeclared.contains_key(&decl.name.text)
        {
            self.lift_local_gen(decl);
            return;
        }
        let (param_tys, ret_ty, fn_ty) = self.local_fn_ty(decl);
        let predeclared = self.cx.predeclared.remove(&decl.name.text);
        let fn_slot = match predeclared {
            Some(slot) => slot,
            None => {
                let fn_slot = self.cx.locals.len();
                self.cx.locals.push(Local {
                    name: decl.name.text.clone(),
                    ty: fn_ty.clone(),
                    binds: None,
                });
                self.cx
                    .scopes
                    .last_mut()
                    .unwrap()
                    .insert(decl.name.text.clone(), fn_slot);
                fn_slot
            }
        };
        // A fn that calls itself reads its own slot, which is set only
        // after its body is lowered: it is captured by reference, so the
        // call finds the fn and not the empty slot.
        self.cx.mut_slots.insert(fn_slot);
        self.lower_local_fn_body(
            decl,
            memo,
            into,
            out,
            param_tys,
            ret_ty,
            fn_ty,
            fn_slot,
            predeclared.is_some(),
        );
    }

    pub(in crate::lower) fn local_fn_ty(&mut self, decl: &ast::FnDecl) -> (Vec<Ty>, Ty, Ty) {
        let param_tys: Vec<Ty> = decl
            .params
            .iter()
            .map(|p| p.ty.as_ref().map(|t| self.resolve_ty(t)).unwrap_or(Ty::Int))
            .collect();
        let ret_ty = decl
            .ret
            .as_ref()
            .map(|r| self.resolve_ty(r))
            .unwrap_or(Ty::Unit);
        // The fn's type says which parameters are `mut`, so a call through
        // its name works on the caller's variable.
        let sig = decl
            .params
            .iter()
            .zip(&param_tys)
            .map(|(p, ty)| match p.mutable {
                true => Ty::Mut(Box::new(ty.clone())),
                false => ty.clone(),
            });
        let fn_ty = Ty::Fn(sig.collect(), Box::new(ret_ty.clone()));
        (param_tys, ret_ty, fn_ty)
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::lower) fn lower_local_fn_body(
        &mut self,
        decl: &ast::FnDecl,
        memo: bool,
        into: Option<&str>,
        out: &mut Vec<Stmt>,
        param_tys: Vec<Ty>,
        ret_ty: Ty,
        fn_ty: Ty,
        fn_slot: usize,
        predeclared: bool,
    ) {
        let mut fresh = FnCx {
            is_closure: true,
            ..FnCx::default()
        };
        fresh.scopes.push(HashMap::new());
        fresh.local_syms.push(HashMap::new());
        fresh.ret = Some(ret_ty.clone());
        let outer_cx = std::mem::replace(&mut self.cx, fresh);
        self.outer_cxs.push(outer_cx);

        let mut param_slots = Vec::new();
        for (p, pty) in decl.params.iter().zip(param_tys.clone()) {
            let slot = self.cx.locals.len();
            self.cx.locals.push(Local {
                name: p.name.text.clone(),
                ty: pty,
                binds: None,
            });
            if p.mutable {
                self.cx.mut_slots.insert(slot);
            }
            self.cx.scopes[0].insert(p.name.text.clone(), slot);
            if let Some(ty_ast) = &p.ty {
                if let Some(rinfo) = self.range_info_from_ast(ty_ast) {
                    self.cx.local_ranges.insert(slot, rinfo);
                }
            }
            param_slots.push(slot);
        }

        let generator = matches!(
            decl.ret.as_ref().map(|t| &t.kind),
            Some(ast::TypeKind::Yields(_))
        );
        let body = if generator {
            // One the calls of another local fn reach first runs to its end.
            let elem_ty = match &ret_ty {
                Ty::List(e) => (**e).clone(),
                _ => Ty::Error,
            };
            let acc = self.new_slot("__yields", ret_ty.clone());
            self.cx.mut_slots.insert(acc);
            let mut stmts = vec![Stmt::Let(
                acc,
                Some(self.mk(ExprKind::Tuple(Vec::new()), ret_ty.clone(), decl.name.span)),
            )];
            self.cx.yield_slot = Some((acc, elem_ty));
            self.gen_body(&decl.body, &mut stmts);
            stmts.push(Stmt::Return(Some(self.local(acc, decl.name.span))));
            Block { stmts }
        } else {
            self.local_fn_block(decl, &ret_ty)
        };

        let inner_cx = std::mem::replace(&mut self.cx, self.outer_cxs.pop().unwrap());
        let cid = self.closures.len();
        self.record_narrow_locals(
            ScopeId::Closure(cid),
            &inner_cx.locals,
            &inner_cx.local_ranges,
        );
        let muts = decl
            .params
            .iter()
            .zip(&param_slots)
            .filter(|(p, _)| p.mutable)
            .map(|(_, slot)| *slot)
            .collect();
        self.closures.push(ClosureDef {
            params: param_slots,
            muts,
            captures: inner_cx.captures,
            locals: inner_cx.locals,
            ret: ret_ty.clone(),
            body,
        });
        let init = self.mk(ExprKind::Closure(cid), fn_ty.clone(), decl.name.span);
        let plain = decl.params.iter().all(|p| !p.mutable);
        if predeclared {
            let place = Place {
                kind: PlaceKind::Local(fn_slot),
                ty: fn_ty,
                span: self.span(decl.name.span),
            };
            out.push(Stmt::Assign(place, AssignOp::Set, init));
            return;
        }
        if !(memo && plain) || matches!(ret_ty, Ty::Unit | Ty::Error) {
            out.push_let(fn_slot, init);
            return;
        }
        // `@memo`: the body is kept under a name of its own, and the fn
        // is a wrapper that looks the arguments up before calling it.
        let span = decl.name.span;
        let body_name = format!("__memo_body{}", self.closures.len());
        let body_slot = self.new_slot(&body_name, fn_ty.clone());
        self.cx
            .scopes
            .last_mut()
            .unwrap()
            .insert(body_name.clone(), body_slot);
        // The body calls the wrapper, so the wrapper's variable comes first.
        out.push(Stmt::Let(fn_slot, None));
        out.push_let(body_slot, init);

        let mut fresh = FnCx {
            is_closure: true,
            ..FnCx::default()
        };
        fresh.scopes.push(HashMap::new());
        fresh.local_syms.push(HashMap::new());
        fresh.ret = Some(ret_ty.clone());
        let outer_cx = std::mem::replace(&mut self.cx, fresh);
        self.outer_cxs.push(outer_cx);
        let mut slots = Vec::new();
        for (p, pty) in decl.params.iter().zip(&param_tys) {
            let slot = self.new_slot(&p.name.text, pty.clone());
            self.cx.scopes[0].insert(p.name.text.clone(), slot);
            slots.push((slot, p.ty.as_ref()));
        }
        let body = self.capture_from_outer(&body_name).expect("just declared");
        let args = slots
            .iter()
            .map(|(slot, _)| Arg::Value(self.local(*slot, span)))
            .collect();
        let call = self.mk_call_expr(self.local(body, span), args, ret_ty.clone(), span);
        let mut lets = Vec::new();
        let stmts = match into {
            Some(table) => self.memo_into_wrapper(table, decl, call, &ret_ty),
            None => self.memo_wrapper(
                memo::MemoHome::Outer,
                &slots,
                &ret_ty,
                call,
                span,
                &mut lets,
            ),
        };
        let inner_cx = std::mem::replace(&mut self.cx, self.outer_cxs.pop().unwrap());
        out.extend(lets);
        let wrapper = self.closures.len();
        self.closures.push(ClosureDef {
            params: slots.iter().map(|(slot, _)| *slot).collect(),
            muts: Vec::new(),
            captures: inner_cx.captures,
            locals: inner_cx.locals,
            ret: ret_ty,
            body: Block { stmts },
        });
        let init = self.mk(ExprKind::Closure(wrapper), fn_ty.clone(), span);
        let place = Place {
            kind: PlaceKind::Local(fn_slot),
            ty: fn_ty,
            span: self.span(span),
        };
        out.push(Stmt::Assign(place, AssignOp::Set, init));
    }

    fn local_fn_block(&mut self, decl: &ast::FnDecl, ret_ty: &Ty) -> Block {
        let tail = if *ret_ty == Ty::Unit {
            Tail::Discard
        } else {
            Tail::Return
        };
        match &decl.body {
            ast::FnBody::Block(block) => self.block(block, tail),
            ast::FnBody::Expr(e) => {
                let mut stmts = Vec::new();
                self.tail(e, tail, &mut stmts);
                Block { stmts }
            }
            ast::FnBody::None => Block::default(),
        }
    }

    /// The domain a one-name lambda parameter indexes an array with, in
    /// the first `a[v]` of its body: what `v` must be.
    pub(in crate::lower) fn indexed_by_param(
        &mut self,
        param: &ast::Pattern,
        body: &ast::Expr,
    ) -> Option<Ty> {
        struct Uses<'n> {
            name: &'n str,
            bases: Vec<(ast::Expr, usize)>,
        }
        impl cpl_syntax::visit::Visitor for Uses<'_> {
            fn expr(&mut self, e: &ast::Expr) {
                if let ast::ExprKind::Index { base, indices } = &e.kind {
                    for (k, index) in indices.iter().enumerate() {
                        if matches!(&index.kind, ast::ExprKind::Name(n) if n == self.name) {
                            self.bases.push(((**base).clone(), k));
                        }
                    }
                }
            }
        }
        let ast::PatternKind::Name(name) = &param.kind else {
            return None;
        };
        let mut uses = Uses {
            name,
            bases: Vec::new(),
        };
        cpl_syntax::visit::Visitor::expr(&mut uses, body);
        cpl_syntax::visit::walk_expr(body, &mut uses);
        uses.bases
            .iter()
            .find_map(|(base, k)| match self.probe_ty_silent(base)? {
                Ty::Array { dims, .. } => match dims.get(*k) {
                    Some(Dim::Pos(d)) => Some(Ty::Pos(*d)),
                    _ => None,
                },
                _ => None,
            })
    }

    pub(in crate::lower) fn ast_ty_has_unbound(
        &self,
        ty: &ast::Type,
        gen_names: &[String],
        env: &GenEnv,
    ) -> bool {
        match &ty.kind {
            ast::TypeKind::Named { name, args } => {
                if args.is_empty()
                    && gen_names.contains(&name.text)
                    && !env.contains_key(&name.text)
                {
                    return true;
                }
                args.iter()
                    .any(|a| self.ast_ty_has_unbound(a, gen_names, env))
            }
            ast::TypeKind::Array { index, elem } => {
                index
                    .iter()
                    .any(|d| self.ast_ty_has_unbound(d, gen_names, env))
                    || self.ast_ty_has_unbound(elem, gen_names, env)
            }
            ast::TypeKind::Optional(t) | ast::TypeKind::Mut(t) | ast::TypeKind::Yields(t) => {
                self.ast_ty_has_unbound(t, gen_names, env)
            }
            ast::TypeKind::Tuple(ts) => ts
                .iter()
                .any(|t| self.ast_ty_has_unbound(t, gen_names, env)),
            ast::TypeKind::Record(fs) => fs
                .iter()
                .any(|(_, t)| self.ast_ty_has_unbound(t, gen_names, env)),
            ast::TypeKind::Fn { params, ret } => {
                params
                    .iter()
                    .any(|t| self.ast_ty_has_unbound(t, gen_names, env))
                    || self.ast_ty_has_unbound(ret, gen_names, env)
            }
            ast::TypeKind::Value(e) => {
                if let ast::ExprKind::Name(n) = &e.kind {
                    gen_names.contains(n) && !env.contains_key(n)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Lowers a lambda expression `(a, b) => body` into a `ClosureDef` with automatic capture resolution.
    pub(in crate::lower) fn lower_lambda(
        &mut self,
        params: &[ast::Pattern],
        ann_tys: &[Option<ast::Type>],
        param_mut: &[bool],
        body: &ast::Expr,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        // `(x, y) => ..` where a one-argument fn of a pair is expected unpacks that pair.
        let pair_arity = match hint {
            Some(Ty::Fn(ps, _)) => match ps.as_slice() {
                [Ty::Tuple(items)] => items.len(),
                _ => 0,
            },
            _ => 0,
        };
        if params.len() > 1 && pair_arity == params.len() && ann_tys.iter().all(Option::is_none) {
            let whole = ast::Pattern {
                kind: ast::PatternKind::Tuple(params.to_vec()),
                span,
            };
            return self.lower_lambda(&[whole], &[None], &[], body, hint, span);
        }
        let (mut param_tys, ret_hint) = match hint {
            Some(Ty::Fn(ps, r)) if ps.len() == params.len() => {
                let rh = if **r == Ty::Error {
                    None
                } else {
                    Some((**r).clone())
                };
                (ps.clone(), rh)
            }
            _ => (vec![Ty::Int; params.len()], None),
        };
        let has_hint = matches!(hint, Some(Ty::Fn(ps, _)) if ps.len() == params.len());
        for (i, ann) in ann_tys.iter().enumerate() {
            if let (Some(ann_ty), Some(slot)) = (ann, param_tys.get_mut(i)) {
                let resolved = self.resolve_ty(ann_ty);
                let is_mut_slot = matches!(&*slot, Ty::Mut(_));
                let slot_base = match &*slot {
                    Ty::Mut(inner) => inner.as_ref(),
                    other => other,
                };
                if has_hint
                    && *slot_base != resolved
                    && *slot_base != Ty::Error
                    && resolved != Ty::Error
                {
                    self.report(
                        ann_ty.span,
                        msgs::ir::TYPE_MISMATCH_LAMBDA_PARAMETER_HAS_TYPE,
                        &[&self.show(&resolved), &self.show(slot_base)],
                    );
                }
                *slot = if is_mut_slot {
                    Ty::Mut(Box::new(resolved))
                } else {
                    resolved
                };
            }
        }

        // Push enclosing FnCx onto `outer_cxs` so lookups inside the lambda can capture variables.
        let mut child_cx = FnCx {
            ret: ret_hint.clone(),
            scopes: vec![HashMap::new()],
            is_closure: true,
            ..FnCx::default()
        };
        let mut param_slots = Vec::with_capacity(params.len());
        let mut mut_slots = Vec::new();
        let mut pre_stmts = Vec::new();
        for (i, ty) in param_tys.iter_mut().enumerate() {
            let pname = match &params[i].kind {
                ast::PatternKind::Name(n) => n.clone(),
                _ => format!("_p{i}"),
            };
            let is_mut = param_mut.get(i).copied().unwrap_or(false) || matches!(&*ty, Ty::Mut(_));
            let local_ty = match &*ty {
                Ty::Mut(inner) => (**inner).clone(),
                other => other.clone(),
            };
            if is_mut && !matches!(&*ty, Ty::Mut(_)) {
                *ty = Ty::Mut(Box::new(local_ty.clone()));
            }
            let slot = child_cx.locals.len();
            child_cx.locals.push(Local {
                name: pname.clone(),
                ty: local_ty,
                binds: None,
            });
            if is_mut {
                child_cx.mut_slots.insert(slot);
                mut_slots.push(slot);
            }
            if matches!(params[i].kind, ast::PatternKind::Name(_)) {
                child_cx.scopes[0].insert(pname, slot);
            }
            param_slots.push(slot);
        }

        let parent_cx = std::mem::replace(&mut self.cx, child_cx);
        self.outer_cxs.push(parent_cx);

        // Destructure any non-name parameter patterns (e.g. `((a, b)) => ...`).
        for (i, pat) in params.iter().enumerate() {
            if !matches!(pat.kind, ast::PatternKind::Name(_)) {
                let pexpr = self.local(param_slots[i], pat.span);
                self.bind(pat, pexpr, &mut pre_stmts);
            }
        }

        let mut body_stmts = pre_stmts;
        self.tail(body, Tail::Return, &mut body_stmts);
        let ret_ty = self.cx.ret.clone().unwrap_or(Ty::Unit);
        if ret_ty != Ty::Unit && !matches!(body_stmts.last(), Some(Stmt::Return(_))) {
            body_stmts.push(Stmt::Trap(
                "closure ended without a value".into(),
                self.span(span),
            ));
        }

        let finished = std::mem::replace(&mut self.cx, self.outer_cxs.pop().expect("parent cx"));
        let cid = self.closures.len();
        self.record_narrow_locals(
            ScopeId::Closure(cid),
            &finished.locals,
            &finished.local_ranges,
        );
        self.closures.push(ClosureDef {
            params: param_slots,
            muts: mut_slots,
            captures: finished.captures,
            locals: finished.locals,
            ret: ret_ty.clone(),
            body: Block { stmts: body_stmts },
        });
        self.mk(
            ExprKind::Closure(cid),
            Ty::Fn(param_tys, Box::new(ret_ty)),
            span,
        )
    }

    /// A named fn as a value. When the expected fn type takes fewer arguments
    /// and the rest have defaults, the value takes only those (`xs.map(yes)`).
    pub(in crate::lower) fn fn_as_closure(
        &mut self,
        fn_id: FnId,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        self.queue_fn(fn_id);
        let all = self.sigs[fn_id].params.clone();
        let keep = match hint {
            Some(Ty::Fn(ps, _))
                if ps.len() < all.len()
                    && all[ps.len()..]
                        .iter()
                        .all(|p| p.default.is_some() && !p.mutable) =>
            {
                ps.len()
            }
            _ => all.len(),
        };
        let params: Vec<Ty> = all[..keep].iter().map(|p| p.ty.clone()).collect();
        let ret = self.ret_of(fn_id, span);
        let mut locals = Vec::new();
        let mut p_slots = Vec::new();
        let mut call_args = Vec::new();
        for (i, ty) in params.iter().enumerate() {
            locals.push(Local {
                name: format!("a{i}"),
                ty: ty.clone(),
                binds: None,
            });
            p_slots.push(i);
            call_args.push(Arg::Value(Expr {
                kind: ExprKind::Local(i),
                ty: ty.clone(),
                span: self.span(span),
            }));
        }
        for p in &all[keep..] {
            let default = p.default.expect("trimmed params have defaults");
            let sm = std::mem::replace(&mut self.cur_mod, self.sigs[fn_id].mod_id);
            let so = std::mem::replace(&mut self.offset, self.sigs[fn_id].offset);
            let se = std::mem::replace(&mut self.gen_env, self.sigs[fn_id].env.clone());
            let value = self.expect(default, &p.ty);
            self.cur_mod = sm;
            self.offset = so;
            self.gen_env = se;
            call_args.push(Arg::Value(value));
        }
        let call = Expr {
            kind: ExprKind::Call(fn_id, call_args),
            ty: ret.clone(),
            span: self.span(span),
        };
        let stmts = if ret == Ty::Unit {
            vec![Stmt::Expr(call)]
        } else {
            vec![Stmt::Return(Some(call))]
        };
        let cid = self.closures.len();
        self.closures.push(ClosureDef {
            params: p_slots,
            muts: Vec::new(),
            captures: Vec::new(),
            locals,
            ret: ret.clone(),
            body: Block { stmts },
        });
        self.mk(ExprKind::Closure(cid), Ty::Fn(params, Box::new(ret)), span)
    }

    /// Captures `self` from an enclosing method into the current closure.
    pub(in crate::lower) fn capture_self_from_outer(&mut self) -> Option<(Slot, StructId)> {
        if !self.cx.is_closure {
            return None;
        }
        let found_level = self
            .outer_cxs
            .iter()
            .rposition(|cx| cx.receiver.is_some())?;
        let (_, sid) = self.outer_cxs[found_level].receiver?;
        let slot = self.capture_from_outer("self")?;
        Some((slot, sid))
    }

    /// Binds a receiver value to an instance method `fn_id`, producing a `Ty::Fn` closure value
    /// (e.g. `g.adj` passed as `V -> Iter<A>`).
    pub(in crate::lower) fn bind_method_closure(
        &mut self,
        fn_id: FnId,
        recv: Expr,
        span: Span,
    ) -> Expr {
        self.queue_fn(fn_id);
        let recv_ty = recv.ty.clone();
        let params: Vec<Ty> = self.sigs[fn_id]
            .params
            .iter()
            .map(|p| p.ty.clone())
            .collect();
        let ret = self.ret_of(fn_id, span);
        let fn_ty = Ty::Fn(params.clone(), Box::new(ret.clone()));
        let build = |lw: &mut Self, outer_recv: usize, is_global: bool| -> Expr {
            let mut locals = Vec::new();
            let mut call_args = Vec::new();
            for (i, ty) in params.iter().enumerate() {
                locals.push(Local {
                    name: format!("a{i}"),
                    ty: ty.clone(),
                    binds: None,
                });
            }
            let inner_recv = locals.len();
            locals.push(Local {
                name: "self".into(),
                ty: recv_ty.clone(),
                binds: None,
            });
            call_args.push(Arg::Value(Expr {
                kind: ExprKind::Local(inner_recv),
                ty: recv_ty.clone(),
                span: lw.span(span),
            }));
            for (i, ty) in params.iter().enumerate() {
                call_args.push(Arg::Value(Expr {
                    kind: ExprKind::Local(i),
                    ty: ty.clone(),
                    span: lw.span(span),
                }));
            }
            let call = Expr {
                kind: ExprKind::Call(fn_id, call_args),
                ty: ret.clone(),
                span: lw.span(span),
            };
            let stmts = if ret == Ty::Unit {
                vec![Stmt::Expr(call)]
            } else {
                vec![Stmt::Return(Some(call))]
            };
            let cid = lw.closures.len();
            lw.closures.push(ClosureDef {
                params: (0..params.len()).collect(),
                muts: Vec::new(),
                captures: vec![Capture {
                    outer: outer_recv,
                    inner: inner_recv,
                    is_global,
                    by_ref: false,
                }],
                locals,
                ret: ret.clone(),
                body: Block { stmts },
            });
            lw.mk(ExprKind::Closure(cid), fn_ty.clone(), span)
        };
        // A variable receiver is captured itself, so a key can lend it.
        if let ExprKind::Local(slot) | ExprKind::Global(slot) = recv.kind {
            return build(self, slot, matches!(recv.kind, ExprKind::Global(_)));
        }
        self.do_expr(Some(&fn_ty), span, |lw, out| {
            let temp = lw.temp(recv_ty.clone());
            out.push_let(temp, recv);
            out.push_give(build(lw, temp, false));
        })
    }
}
