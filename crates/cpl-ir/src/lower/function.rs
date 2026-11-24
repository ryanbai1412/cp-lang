//! Lowering fn bodies: the queue of fns the program reaches, each body once,
//! and return types read from the body.

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn pattern_names(pat: &ast::Pattern, out: &mut HashSet<String>) {
        match &pat.kind {
            ast::PatternKind::Name(name) => {
                out.insert(name.clone());
            }
            ast::PatternKind::Domain(name) => {
                out.insert(name.text.clone());
            }
            _ => pat.parts().for_each(|p| Self::pattern_names(p, out)),
        }
    }

    pub(crate) fn queue_fn(&mut self, id: FnId) {
        if self.queued_set.insert(id) {
            self.sigs[id].asked_at = self.asked_at.callee(self.cur_mod);
            self.queued_fns.push(id);
        }
    }

    /// Lowers the body of fn `id`, once.
    pub(in crate::lower) fn function(&mut self, id: FnId) {
        if self.sigs[id].state != State::Pending {
            return;
        }
        self.ensure_sig(id, None);
        // Skip uncalled `@test` fns in non-test runs unless explicitly referenced.
        self.sigs[id].state = State::InProgress;
        let dom_start = self.domains.len();
        let decl = self.sigs[id].decl;
        let is_gen = self.sigs[id].is_generator;
        let new_doms = self.sigs[id].new_domain_slots.clone();
        let saved_mod = std::mem::replace(&mut self.cur_mod, self.sigs[id].mod_id);
        let saved_offset = std::mem::replace(&mut self.offset, self.sigs[id].offset);
        let saved_env = std::mem::replace(&mut self.gen_env, self.sigs[id].env.clone());
        let saved_self = std::mem::replace(&mut self.self_struct, self.sigs[id].owner);
        let saved_asked = std::mem::replace(&mut self.asked_at, self.sigs[id].asked_at.clone());
        let saved_fn = self.growth.cur.replace(id);

        let mut cx = FnCx {
            ret: self.sigs[id].ret.clone(),
            scopes: vec![HashMap::new()],
            local_syms: vec![HashMap::new()],
            ..FnCx::default()
        };
        let mut params = Vec::new();
        if let (Some(mutable), Some(owner)) = (self.sigs[id].receiver, self.sigs[id].owner) {
            cx.locals.push(Local {
                name: "self".into(),
                ty: Ty::Struct(owner),
                binds: None,
            });
            cx.scopes[0].insert("self".into(), 0);
            if mutable {
                cx.mut_slots.insert(0);
            }
            cx.receiver = Some((0, owner));
            params.push(ParamDef {
                mutable,
                range: None,
            });
        }
        for param in &self.sigs[id].params {
            let slot = cx.locals.len();
            cx.scopes[0].insert(param.name.clone(), slot);
            if param.mutable {
                cx.mut_slots.insert(slot);
            }
            cx.locals.push(Local {
                name: param.name.clone(),
                ty: param.ty.clone(),
                binds: None,
            });
            params.push(ParamDef {
                mutable: param.mutable,
                range: None,
            });
        }
        for ast_p in &decl.params {
            let bare = ast_p.ty.as_ref().map(|t| match &t.kind {
                ast::TypeKind::Optional(inner) => &**inner,
                _ => t,
            });
            if let Some(ast::Type {
                kind: ast::TypeKind::Named { name, args },
                ..
            }) = bare
            {
                if args.is_empty() {
                    if let Some(GenArg::Op(op)) = self.gen_env.get(&name.text) {
                        cx.op_locals.insert(ast_p.name.text.clone(), *op);
                    }
                }
            }
            let op_key = mono::op_param_key(&ast_p.name.text);
            if let Some(GenArg::Op(op)) = self.gen_env.get(&op_key) {
                cx.op_locals.insert(ast_p.name.text.clone(), *op);
            }
        }
        if let Some(gid) = self.sigs[id]
            .owner
            .and_then(|sid| self.structs[sid].origin_gen.as_ref().map(|(g, _)| *g))
        {
            for f in &self.gen_structs[gid].decl.fields {
                if let ast::TypeKind::Named { name, args } = &f.ty.kind {
                    if let (true, Some(GenArg::Op(op))) =
                        (args.is_empty(), self.gen_env.get(&name.text))
                    {
                        cx.op_locals.entry(f.name.text.clone()).or_insert(*op);
                    }
                }
            }
        }
        let saved_cx = std::mem::replace(&mut self.cx, cx);
        self.bind_param_facts(decl, &mut params);
        if self.sigs[id].mod_id == 0
            && self.sigs[id].owner.is_none()
            && self.sigs[id].params.iter().all(|p| p.ty.crosses_to_judge())
            && !matches!(&self.sigs[id].ret, Some(ret) if !ret.crosses_to_judge())
        {
            let declared = (0..decl.params.len())
                .map(|slot| self.cx.local_ranges.get(&slot).cloned())
                .collect();
            let ret = decl.ret.as_ref().and_then(|t| self.range_info_from_ast(t));
            self.sig_ranges.insert(id, (declared, ret));
        }
        let mut out = Vec::new();

        // If this is a constructor/static method of a struct with owned domains (e.g. `Rooted<V>.Pre`),
        // initialize those owned domains at the start of the method.
        // A method on a value sets the domains sized from its fields.
        if let Some(owner) = self.sigs[id].owner {
            let receiver = self.sigs[id].receiver.is_some();
            let inits = self.structs[owner].owned_domain_inits.clone();
            for init in inits.into_iter().filter(|i| i.per_value == receiver) {
                let sm = std::mem::replace(&mut self.cur_mod, init.mod_id);
                let so = std::mem::replace(&mut self.offset, init.offset);
                self.init_domain_id(init.id, init.def, &mut out);
                self.cur_mod = sm;
                self.offset = so;
            }
        }

        // A generator (`-> yields T`) builds its values in a `List<T>` accumulator; `= e`
        // yields each element of `e`, so it feeds a loop as a block body does.
        let is_gen = is_gen
            && match &decl.body {
                ast::FnBody::Block(_) => true,
                ast::FnBody::Expr(e) => !collect::is_cpp(e),
                ast::FnBody::None => false,
            };
        if is_gen {
            let list_ty = self.cx.ret.clone().unwrap_or_else(|| Ty::list(Ty::Error));
            let elem_ty = match &list_ty {
                Ty::List(e) => (**e).clone(),
                _ => Ty::Error,
            };
            let acc = self.new_slot("__yields", list_ty.clone());
            self.cx.mut_slots.insert(acc);
            out.push(Stmt::Let(
                acc,
                Some(self.mk(ExprKind::Tuple(Vec::new()), list_ty, decl.name.span)),
            ));
            self.cx.yield_slot = Some((acc, elem_ty));
        }

        // If this fn returns `(domain C, ...)`, pre-bind each fresh domain as a local domain symbol.
        for (dname, dom_id) in &new_doms {
            self.cx.local_syms[0].insert(dname.clone(), Sym::Domain(*dom_id));
            self.cx.new_domain_slots.push(*dom_id);
        }

        let saved_fn_contexts = self.contexts.split_off(self.file_contexts_len);
        let saved_fn_io_sep = self.io_sep.take();
        let saved_fn_io_prec = self.io_precision.take();
        let saved_fn_unchecked = std::mem::replace(&mut self.unchecked, 0);
        let saved_fn_overflow = std::mem::replace(&mut self.overflow_checks_off, 0);
        self.cx.uses_io = decl.uses.iter().any(|u| u.text == "io");
        self.cx.lexical_uses = Some(decl.uses.iter().map(|u| u.text.clone()).collect());
        match &decl.body {
            ast::FnBody::Block(block) => {
                let tail = if is_gen { Tail::Discard } else { Tail::Return };
                let body = self.block(block, tail);
                out.extend(body.stmts);
            }
            // `-> yields T = e` yields each element of `e`.
            // `= e` walks `e` and yields each element, whatever sequence `e` is.
            ast::FnBody::Expr(expr) if is_gen => {
                let span = expr.span;
                let stmt = |kind| ast::Stmt {
                    kind,
                    span,
                    decorators: Vec::new(),
                    public: false,
                };
                let x = "__yielded".to_string();
                let each = stmt(ast::StmtKind::For {
                    label: None,
                    pat: ast::Pattern {
                        kind: ast::PatternKind::Name(x.clone()),
                        span,
                    },
                    iter: expr.clone(),
                    body: ast::Block {
                        stmts: vec![stmt(ast::StmtKind::Yield {
                            value: ast::Expr {
                                kind: ast::ExprKind::Name(x),
                                span,
                            },
                            from: false,
                        })],
                        span,
                    },
                    else_body: None,
                });
                self.stmt(&each, &mut out);
            }
            ast::FnBody::Expr(e) if !collect::is_cpp(e) => self.tail(e, Tail::Return, &mut out),
            // The judge's grader, or an `@cpp` body's C++, supplies the
            // body: the signature is all there is.
            ast::FnBody::Expr(_) | ast::FnBody::None => {}
        }
        self.fn_keys(decl, &mut out);
        self.contexts.truncate(self.file_contexts_len);
        self.contexts.extend(saved_fn_contexts);
        self.io_sep = saved_fn_io_sep;
        self.io_precision = saved_fn_io_prec;
        self.unchecked = saved_fn_unchecked;
        self.overflow_checks_off = saved_fn_overflow;

        if let Some((acc, _)) = self.cx.yield_slot {
            let res = self.local(acc, decl.name.span);
            out.push(Stmt::Return(Some(res)));
        }

        let ret = self.cx.ret.clone().unwrap_or(Ty::Unit);
        if ret != Ty::Unit && !matches!(out.last(), Some(Stmt::Return(_))) {
            out.push(Stmt::Trap(
                format!("fn `{}` ended without a value", decl.name.text),
                self.span(decl.name.span),
            ));
        }
        if self.sigs[id].assumed_unit && !matches!(ret, Ty::Unit | Ty::Error) {
            // Its own calls were read as giving nothing, but its last
            // line gives a value.
            let name = decl.name.text.clone();
            self.error(
                codes::UNKNOWN_TYPE,
                decl.name.span,
                format!("`{name}` calls itself, so it needs its return type written"),
            );
            let shown = self.show(&ret);
            self.help(format!("add `-> {shown}` to `fn {name}`"));
        }
        // `@memo`: the body moves to a fn of its own, and this one looks
        // the arguments up before calling it.
        let plain = params.iter().all(|p| !p.mutable) && self.sigs[id].receiver.is_none();
        if self.memo_fns.contains(&id) && plain && !is_gen && !matches!(ret, Ty::Unit | Ty::Error) {
            let wrapper = FnCx {
                ret: Some(ret.clone()),
                scopes: vec![HashMap::new()],
                local_syms: vec![HashMap::new()],
                ..FnCx::default()
            };
            let body_cx = std::mem::replace(&mut self.cx, wrapper);
            let body_id = self.sigs.len();
            self.sigs.push(FnSig {
                name: format!("{}_body", self.sigs[id].name),
                params: Vec::new(),
                ret: Some(ret.clone()),
                assumed_unit: false,
                owner: None,
                receiver: None,
                mod_id: self.sigs[id].mod_id,
                env: self.sigs[id].env.clone(),
                decl,
                offset: self.sigs[id].offset,
                is_generator: false,
                new_domain_slots: Vec::new(),
                returns_fresh_domain: false,
                used_fresh_call: false,
                state: State::Done,
                sig: SigState::Resolved,
                asked_at: self.asked_at.callee(self.cur_mod),
            });
            self.bodies.push(Some(Function {
                name: format!("{}_body", self.sigs[id].name),
                params: params.clone(),
                ret: ret.clone(),
                locals: body_cx.locals,
                body: Block {
                    stmts: std::mem::take(&mut out),
                },
            }));
            let mut slots = Vec::new();
            for (param, written) in self.sigs[id].params.iter().zip(&decl.params) {
                let slot = self.cx.locals.len();
                self.cx.scopes[0].insert(param.name.clone(), slot);
                self.cx.locals.push(Local {
                    name: param.name.clone(),
                    ty: param.ty.clone(),
                    binds: None,
                });
                slots.push((slot, written.ty.as_ref()));
            }
            let args = slots
                .iter()
                .map(|(slot, _)| Arg::Value(self.local(*slot, decl.name.span)))
                .collect();
            let call = self.mk(ExprKind::Call(body_id, args), ret.clone(), decl.name.span);
            out = match self.memo_into.get(&id).cloned() {
                Some(table) => self.memo_into_wrapper(&table, decl, call, &ret),
                None => self.memo_wrapper(
                    memo::MemoHome::Global,
                    &slots,
                    &ret,
                    call,
                    decl.name.span,
                    &mut Vec::new(),
                ),
            };
            // A flat table is read by a short fn the C++ compiler puts
            // inline at each call; filling a cell is a fn of its own.
            if let Some(fast) = self.memo_fast.take() {
                let fresh = FnCx {
                    ret: Some(ret.clone()),
                    scopes: vec![HashMap::new()],
                    local_syms: vec![HashMap::new()],
                    ..FnCx::default()
                };
                let slow_cx = std::mem::replace(&mut self.cx, fresh);
                let slow_id = self.sigs.len();
                self.sigs.push(FnSig {
                    name: format!("{}_fill", self.sigs[id].name),
                    params: Vec::new(),
                    ret: Some(ret.clone()),
                    assumed_unit: false,
                    owner: None,
                    receiver: None,
                    mod_id: self.sigs[id].mod_id,
                    env: self.sigs[id].env.clone(),
                    decl,
                    offset: self.sigs[id].offset,
                    is_generator: false,
                    new_domain_slots: Vec::new(),
                    returns_fresh_domain: false,
                    used_fresh_call: false,
                    state: State::Done,
                    sig: SigState::Resolved,
                    asked_at: self.asked_at.callee(self.cur_mod),
                });
                self.bodies.push(Some(Function {
                    name: format!("{}_fill", self.sigs[id].name),
                    params: params.clone(),
                    ret: ret.clone(),
                    locals: slow_cx.locals,
                    body: Block {
                        stmts: std::mem::take(&mut out),
                    },
                }));
                for param in &self.sigs[id].params {
                    self.cx.locals.push(Local {
                        name: param.name.clone(),
                        ty: param.ty.clone(),
                        binds: None,
                    });
                }
                let args = (0..params.len())
                    .map(|slot| Arg::Value(self.local(slot, decl.name.span)))
                    .collect();
                let fill = self.mk(ExprKind::Call(slow_id, args), ret.clone(), decl.name.span);
                out = self.memo_lookup(fast, fill, &ret, decl.name.span);
            }
        }
        if is_narrowable_elem_ty(&ret) {
            if let Some(ret_ty) = decl.ret.as_ref() {
                if let Some(layout) = self
                    .range_info_from_ast(ret_ty)
                    .and_then(|r| r.narrow_elem_layout(&ret))
                {
                    self.narrow_elems.insert(ElemLoc::FnRet(id), layout);
                }
            }
        }
        let cx = std::mem::replace(&mut self.cx, saved_cx);
        self.record_narrow_locals(ScopeId::Fn(id), &cx.locals, &cx.local_ranges);
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
        self.asked_at = saved_asked;
        self.growth.cur = saved_fn;
        if self.domains.len() > dom_start {
            let mut fresh = Vec::new();
            self.fresh_domains_in(
                &ret,
                dom_start,
                &mut HashSet::new(),
                &mut HashSet::new(),
                &mut fresh,
            );
            self.sigs[id].returns_fresh_domain = !fresh.is_empty();
            // `fn wrap(a): compress(a)` with no `->`: the `domain` slots of
            // the tuple it returns are the fresh domains its body made.
            let binders = match &ret {
                Ty::Tuple(ts) => ts.iter().filter(|t| **t == Ty::DomainVal).count(),
                _ => 0,
            };
            if self.sigs[id].new_domain_slots.is_empty() && binders > 0 && binders == fresh.len() {
                self.sigs[id].new_domain_slots = fresh
                    .into_iter()
                    .map(|d| (self.domains[d].clone(), d))
                    .collect();
            }
        }
        self.sigs[id].ret = Some(ret.clone());
        self.sigs[id].state = State::Done;
        self.bodies[id] = Some(Function {
            name: self.sigs[id].name.clone(),
            params,
            ret,
            locals: cx.locals,
            body: Block { stmts: out },
        });
    }

    /// The return type of fn `id`, reading its body now if it has no `->`.
    pub(in crate::lower) fn ret_of(&mut self, id: FnId, span: Span) -> Ty {
        if let Some(ty) = &self.sigs[id].ret {
            return ty.clone();
        }
        if self.sigs[id].state == State::InProgress {
            // A fn that calls itself and never writes `return value` is
            // a procedure: its calls give nothing.
            let decl = self.sigs[id].decl;
            if matches!(&decl.body, ast::FnBody::Block(block) if !returns_a_value(block)) {
                self.sigs[id].ret = Some(Ty::Unit);
                self.sigs[id].assumed_unit = true;
                return Ty::Unit;
            }
            let name = self.sigs[id].decl.name.text.clone();
            self.error(
                codes::UNKNOWN_TYPE,
                span,
                format!("`{name}` calls itself, so it needs its return type written"),
            );
            self.help(format!(
                "add `-> Int`, or the type it returns, to `fn {name}`"
            ));
            return Ty::Error;
        }
        self.function(id);
        self.sigs[id].ret.clone().unwrap_or(Ty::Error)
    }
}
