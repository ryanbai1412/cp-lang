//! The methods `Rev<S>` declares (`a.rev.take(k)`), called on a list view
//! known to walk backwards: each runs its std body inline (an expression, or
//! a block giving its last value), with `self` the view and `self.src` the
//! sequence it reverses.

use super::*;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn rev_method(
        &mut self,
        value: Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        span: Span,
    ) -> Result<Expr, Expr> {
        if !matches!(value.ty, Ty::ListView(..)) || !self.is_reversed(&value) {
            return Err(value);
        }
        let Some(gid) = self.view_struct("Rev") else {
            return Err(value);
        };
        let decl = self.gen_structs[gid].decl;
        let block_method = |stmt: &'a ast::Stmt| match &stmt.kind {
            ast::StmtKind::Fn(f) if f.name.text == name.text => match &f.body {
                ast::FnBody::Expr(body) if body.leaves() => None,
                ast::FnBody::Expr(_) | ast::FnBody::Block(_) => Some((f, &f.body)),
                ast::FnBody::None => None,
            },
            _ => None,
        };
        // A method the struct declares types its own `self`; one in an
        // `impl Rev<[D]T>:` block takes the impl's target, and its names.
        let in_struct = decl.methods.iter().find_map(block_method).map(|(f, body)| {
            let t = &self.gen_structs[gid];
            let receiver = f.params.first().and_then(|p| p.ty.as_ref());
            (f, body, t.mod_id, t.offset, receiver, Vec::new())
        });
        let in_impl = || {
            self.impls.iter().find_map(|imp| {
                let on_view = matches!(&imp.target.kind,
                    ast::TypeKind::Named { name, .. } if name.text == decl.name.text);
                let (f, body) = imp
                    .body
                    .stmts
                    .iter()
                    .filter(|_| on_view)
                    .find_map(block_method)?;
                let names = self.impl_params(imp.mod_id, imp.target);
                Some((f, body, imp.mod_id, imp.offset, Some(imp.target), names))
            })
        };
        let Some((f, body, tpl_mod, tpl_offset, receiver, impl_names)) = in_struct.or_else(in_impl)
        else {
            return Err(value);
        };
        let params: Vec<&ast::Param> = f.params.iter().skip(1).collect();
        let gen_names: Vec<String> = (f.generics.iter().map(|g| g.text.clone()))
            .chain(impl_names)
            .collect();
        let generic = |t: &ast::Type| gen_names.iter().any(|g| mono::ty_mentions(t, g));
        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
        let mut tys: Vec<Option<Ty>> = params
            .iter()
            .map(|p| {
                p.ty.as_ref()
                    .filter(|t| !generic(t))
                    .map(|t| self.resolve_ty(t))
            })
            .collect();
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        // The method's own generics bind from its arguments, as a call of a
        // generic fn does: `f: M` given `+` makes `M` that op.
        let mut env = GenEnv::new();
        // `self: Rev<[D]T>` binds `D` and `T` from the sequence the view reverses.
        if let Some(ast::Type {
            kind: ast::TypeKind::Named { name, args: of },
            ..
        }) = receiver
        {
            if let (true, [source]) = (name.text == decl.name.text, of.as_slice()) {
                self.unify_ast_ty(source, &value.ty, tpl_mod, &gen_names, &mut env);
                // A list is keyed by its own positions: `D` is a domain over
                // them, as when a list is given for a generic fn's `[D]T`.
                if let ast::TypeKind::Array { index, .. } = &source.kind {
                    if let [ast::Type {
                        kind: ast::TypeKind::Named { name: key, args },
                        ..
                    }] = index.as_slice()
                    {
                        if args.is_empty() && env.get(&key.text) == Some(&GenArg::Ty(Ty::Int)) {
                            let d = self.range_domain_at(&key.text, 0, true, span, None);
                            env.insert(key.text.clone(), GenArg::Domain(d));
                        }
                    }
                }
            }
        }
        let mut ops: Vec<(String, BareOp)> = Vec::new();
        let mut given = Vec::with_capacity(params.len());
        for (pos, (p, ty)) in params.iter().zip(&tys).enumerate() {
            let arg = args
                .iter()
                .find(|a| a.name.as_ref().is_some_and(|n| n.text == p.label_text()))
                .or_else(|| args.get(pos).filter(|a| a.name.is_none()));
            let bare_gen = p.ty.as_ref().and_then(|t| match &t.kind {
                ast::TypeKind::Named { name, args } if args.is_empty() => {
                    gen_names.iter().find(|g| **g == name.text)
                }
                _ => None,
            });
            let op = arg.zip(bare_gen).and_then(|(a, g)| {
                let at = (tpl_mod, tpl_offset, &env);
                self.bare_op_for(&f.bounds, g, &a.value, at)
                    .map(|op| (g, op))
            });
            if let Some((g, op)) = op {
                if let GenArg::Op(op) = op {
                    ops.push((p.name.text.clone(), op));
                }
                env.insert(g.clone(), op);
                given.push(None);
                continue;
            }
            let lowered = arg.map(|a| match ty {
                Some(ty) => self.expect(&a.value, ty),
                None => self.expr(&a.value, None),
            });
            if let (Some(pat), Some(arg)) = (p.ty.as_ref().filter(|t| generic(t)), &lowered) {
                self.unify_ast_ty(pat, &arg.ty, tpl_mod, &gen_names, &mut env);
            }
            given.push(lowered);
        }
        let saved_mod = std::mem::replace(&mut self.cur_mod, tpl_mod);
        let saved_offset = std::mem::replace(&mut self.offset, tpl_offset);
        let saved_env = std::mem::replace(&mut self.gen_env, env);
        for (p, ty) in params.iter().zip(tys.iter_mut()) {
            if let Some(t) = p.ty.as_ref().filter(|t| generic(t)) {
                let bound = gen_names
                    .iter()
                    .filter(|g| mono::ty_mentions(t, g))
                    .all(|g| self.gen_env.contains_key(g));
                *ty = bound.then(|| self.resolve_ty(t));
            }
        }
        let saved_self = self.self_struct.take();
        let saved_ops = self.cx.op_locals.clone();
        self.cx.scopes.push(HashMap::new());
        let this = self.temp(value.ty.clone());
        self.cx.view_rev.insert(this);
        self.cx.pending.push(Stmt::Let(this, Some(value)));
        self.bind_local("self", this);
        let op_params: Vec<String> = ops.iter().map(|(name, _)| name.clone()).collect();
        for (name, op) in ops {
            self.cx.op_locals.insert(name, op);
        }
        for ((p, ty), arg) in params.iter().zip(&tys).zip(given) {
            if op_params.contains(&p.name.text) {
                continue;
            }
            let arg = match (arg, &p.default) {
                (Some(arg), _) => arg,
                (None, Some(default)) => match ty {
                    Some(ty) => self.expect(default, ty),
                    None => self.expr(default, None),
                },
                (None, None) => self.err_expr(span),
            };
            let slot = self.temp(arg.ty.clone());
            self.cx.pending.push(Stmt::Let(slot, Some(arg)));
            self.bind_local(&p.name.text, slot);
        }
        let out = match body {
            ast::FnBody::Block(block) => {
                let tail = if f.ret.is_some() {
                    Tail::Give
                } else {
                    Tail::Discard
                };
                self.do_expr(None, span, |lw, out| {
                    out.extend(lw.block(block, tail).stmts)
                })
            }
            ast::FnBody::Expr(body) => self.expr(body, None),
            ast::FnBody::None => self.err_expr(span),
        };
        self.cx.scopes.pop();
        self.cx.op_locals = saved_ops;
        self.cur_mod = saved_mod;
        self.offset = saved_offset;
        self.gen_env = saved_env;
        self.self_struct = saved_self;
        Ok(out)
    }

    /// The std struct `name` that types a built-in view, wherever the
    /// caller's module stands.
    pub(in crate::lower) fn view_struct(&self, name: &str) -> Option<GenStructId> {
        let found = |m: ModId| match self.mod_syms[m].get(name) {
            Some(Sym::GenStruct(gid)) => Some(*gid),
            _ => None,
        };
        self.std_mod.into_iter().find_map(found).or_else(|| {
            self.mods
                .iter()
                .flat_map(|a| a.modules())
                .filter(|module| module.name.starts_with("std."))
                .filter_map(|module| self.path_to_mod.get(&module.path).copied())
                .find_map(found)
        })
    }

    fn bind_local(&mut self, name: &str, slot: Slot) {
        self.cx
            .scopes
            .last_mut()
            .expect("a scope")
            .insert(name.to_string(), slot);
    }
}
