//! An impl head's inline bounds (R12.3): `impl Seq<T> for Wrap<S: Seq<T>>`
//! takes `T` from what `S` is, and gives `Wrap<S>` nothing where `S` is not a
//! `Seq`; a bound on any trait, `S: Versioned<D, T>`, binds `D` and `T` from
//! `S`'s impl of it.

use super::*;

impl GenArg {
    /// What a key dimension binds a key generic to: its domain, or `Int` for a plain index.
    pub(super) fn of_key(dim: Dim) -> Option<GenArg> {
        match dim {
            Dim::Pos(d) => Some(GenArg::Domain(d)),
            Dim::Int | Dim::IntFrom(_) => Some(GenArg::Ty(Ty::Int)),
            _ => None,
        }
    }
}

impl<'a> Lowerer<'a> {
    /// Binds the names an impl's bounds leave open from the types its target
    /// bound: `K` and `T` in `Z<A: Indexed<K, T>>` are `A`'s key and element.
    /// False when a bound does not hold, so the impl does not apply.
    pub(super) fn bind_impl_bounds(
        &mut self,
        bounds: &[ast::Bound],
        m: ModId,
        env: &mut GenEnv,
    ) -> bool {
        let mut open = Vec::new();
        for tr in bounds.iter().flat_map(|b| &b.traits) {
            for name in self.impl_params(m, tr) {
                if !env.contains_key(&name) && !open.contains(&name) {
                    open.push(name);
                }
            }
        }
        for b in bounds {
            let Some(GenArg::Ty(seq)) = env.get(&b.name.text).cloned() else {
                continue;
            };
            for tr in &b.traits {
                let ast::TypeKind::Named { name, args } = &tr.kind else {
                    continue;
                };
                let trait_name = name.text.as_str();
                let (key, elem) = match trait_name {
                    "Seq" | "Bidir" | "Sized" | "Positional" => (None, args.first()),
                    "Keyed" | "Indexed" | "IndexedMut" => (args.first(), args.get(1)),
                    _ => {
                        let known = self.known_trait_args(args, m, env);
                        if let Some((given, opened)) =
                            self.impl_trait_args(&seq, trait_name, &known)
                        {
                            env.extend(opened);
                            for (arg, ty) in args.iter().zip(&given) {
                                self.unify_ast_ty(arg, ty, m, &open, env);
                            }
                        }
                        continue;
                    }
                };
                let builtin = match trait_name {
                    "Indexed" | "Bidir" | "Positional" => seq.is_indexed(),
                    "IndexedMut" => seq.is_indexed_mut(),
                    _ => seq.is_seq(),
                };
                if !builtin && !(matches!(seq, Ty::Struct(_)) && self.implements(&seq, trait_name))
                {
                    return false;
                }
                if let Some(k) = key {
                    let key_ty = match self.seq_key_of(&seq) {
                        Some(Dim::Pos(d)) => Ty::Pos(d),
                        _ => Ty::Int,
                    };
                    self.unify_ast_ty(k, &key_ty, m, &open, env);
                }
                if let (Some(t), Some(e)) = (elem, self.seq_elem_of(&seq)) {
                    self.unify_ast_ty(t, &e, m, &open, env);
                }
            }
        }
        true
    }

    /// What walking a value of `ty` gives: a built-in sequence's element, or what the `iter`
    /// of a struct's `impl Seq` yields.
    pub(super) fn seq_elem_of(&mut self, ty: &Ty) -> Option<Ty> {
        if let Some(e) = ty.seq_elem() {
            return Some(e);
        }
        let Ty::Struct(sid) = ty else {
            return None;
        };
        let f = *self.structs[*sid].methods.get("iter")?;
        if !self.sigs[f].params.is_empty() {
            return None;
        }
        self.queue_fn(f);
        match self.ret_of(f, Span::default()) {
            Ty::List(elem) => Some(*elem),
            _ => None,
        }
    }

    /// What a value of `ty` is keyed by: a built-in sequence's key, or the key the `items`
    /// of a struct's `impl Keyed` yields.
    pub(super) fn seq_key_of(&mut self, ty: &Ty) -> Option<Dim> {
        if let Some(k) = ty.seq_key() {
            return Some(k);
        }
        let Ty::Struct(sid) = ty else {
            return None;
        };
        let f = *self.structs[*sid].methods.get("items")?;
        if !self.sigs[f].params.is_empty() {
            return None;
        }
        self.queue_fn(f);
        let Ty::List(pair) = self.ret_of(f, Span::default()) else {
            return None;
        };
        match &*pair {
            Ty::Tuple(parts) if parts.len() == 2 => match parts[0] {
                Ty::Pos(d) => Some(Dim::Pos(d)),
                Ty::Int => Some(Dim::Int),
                _ => None,
            },
            _ => None,
        }
    }

    /// A struct value with an `impl Seq` and no `Show` of its own, as the list its `iter`
    /// walks: any `Seq` prints as a sequence (R10.3.1).
    pub(super) fn shown_struct(&mut self, value: Expr) -> Expr {
        let Ty::Struct(sid) = value.ty else {
            return value;
        };
        if self.structs[sid].def.show_fn.is_some() || !self.implements(&value.ty, "Seq") {
            return value;
        }
        let Some(&f) = self.structs[sid].methods.get("iter") else {
            return value;
        };
        if !self.sigs[f].params.is_empty() {
            return value;
        }
        self.queue_fn(f);
        match self.ret_of(f, Span::default()) {
            ret @ Ty::List(_) => {
                let span = value.span;
                Expr {
                    kind: ExprKind::Call(f, vec![Arg::Value(value)]),
                    ty: ret,
                    span,
                }
            }
            _ => value,
        }
    }

    /// The impl's type parameters bound for the concrete struct `sid`, if impl `idx` is
    /// for `sid` and its bounds hold there (`impl Indexed for Mapped<S: Indexed>`).
    /// The trait arguments a struct's own `impl` of the trait `tr` names gives,
    /// e.g. `(Int, (Int, Int))` for `Indexed` on `Items<List<Int>>`.
    pub(super) fn struct_trait_args(&mut self, ty: &Ty, tr: &ast::Type) -> Option<Vec<Ty>> {
        let (Ty::Struct(sid), ast::TypeKind::Named { name, args }) = (ty, &tr.kind) else {
            return None;
        };
        if args.is_empty() {
            return None;
        }
        for idx in 0..self.impls.len() {
            let imp = &self.impls[idx];
            if imp.trait_name.as_deref() != Some(name.text.as_str())
                || imp.trait_args.len() != args.len()
            {
                continue;
            }
            let (m, offset, trait_args) = (imp.mod_id, imp.offset, imp.trait_args.clone());
            let Some(env) = self.impl_env_for(idx, *sid) else {
                continue;
            };
            let given = self.quietly_in(m, offset, env, |l| {
                trait_args
                    .iter()
                    .map(|t| l.resolve_ty(t))
                    .collect::<Vec<_>>()
            });
            if !given.contains(&Ty::Error) {
                return Some(given);
            }
        }
        None
    }

    pub(super) fn impl_env_for(&mut self, idx: usize, sid: StructId) -> Option<GenEnv> {
        let sdecl = self.structs[sid].decl?;
        let struct_decl_name = sdecl.name.text.clone();
        let struct_mod = self.structs[sid].mod_id;
        let origin_args: Vec<GenArg> = self.structs[sid]
            .origin_gen
            .as_ref()
            .map(|(_, a)| a.clone())
            .unwrap_or_default();
        let imp_mod = self.impls[idx].mod_id;
        let imp_target = self.impls[idx].target;
        let ast::TypeKind::Named {
            name: tname,
            args: targs,
        } = &imp_target.kind
        else {
            return None;
        };
        // Check whether `tname` in `imp_mod` refers to the same struct.
        let keyed = cpl_check::arity_key(&tname.text, targs.len());
        let syms = &self.mod_syms[imp_mod];
        let target_matches = match syms.get(&keyed).or_else(|| syms.get(&tname.text)).copied() {
            Some(Sym::Struct(id)) => id == sid,
            Some(Sym::GenStruct(gid)) => self.structs[sid]
                .origin_gen
                .as_ref()
                .is_some_and(|(g, _)| *g == gid),
            _ => tname.text == struct_decl_name && imp_mod == struct_mod,
        };
        if !target_matches {
            return None;
        }
        if targs.len() != origin_args.len() {
            return None;
        }
        // Bind impl type params from `targs` vs `origin_args`.
        let mut env = self.structs[sid].env.clone();
        let mut ok = true;
        for (targ, actual) in targs.iter().zip(&origin_args) {
            match &targ.kind {
                ast::TypeKind::Named { name, args } if args.is_empty() => {
                    env.insert(name.text.clone(), actual.clone());
                }
                ast::TypeKind::Value(expr) => {
                    if let ast::ExprKind::Name(n) = &expr.kind {
                        env.insert(n.clone(), actual.clone());
                    } else if let Some(v) = self.eval_const_int(expr) {
                        if *actual != GenArg::Int(v) {
                            ok = false;
                        }
                    }
                }
                _ => {}
            }
        }
        (ok && self.bind_impl_bounds(self.impls[idx].bounds, imp_mod, &mut env)).then_some(env)
    }

    /// The generic env impl `idx` binds when its head names the struct `sid`:
    /// `impl Next<V> for Graph<V, A>` on `Graph<X, Y>` binds `V = X`, `A = Y`.
    pub(super) fn struct_impl_env(&self, idx: usize, sid: StructId) -> Option<GenEnv> {
        let (m, target) = (self.impls[idx].mod_id, self.impls[idx].target);
        let ast::TypeKind::Named {
            name: head,
            args: targs,
        } = &target.kind
        else {
            return None;
        };
        let origin = self.structs[sid].origin_gen.as_ref();
        let fits = match self.mod_syms[m].get(&head.text).copied() {
            Some(Sym::Struct(id)) => id == sid,
            Some(Sym::GenStruct(g)) => origin.is_some_and(|(o, _)| *o == g),
            _ => false,
        };
        if !fits {
            return None;
        }
        let open = self.impl_params(m, target);
        let mut env = self.structs[sid].env.clone();
        let given = origin.map_or(&[][..], |(_, a)| &a[..]);
        for (targ, arg) in targs.iter().zip(given) {
            match &targ.kind {
                ast::TypeKind::Named { name, args }
                    if args.is_empty() && open.contains(&name.text) =>
                {
                    env.insert(name.text.clone(), arg.clone());
                }
                _ => {}
            }
        }
        Some(env)
    }

    /// The args `ty`'s impl of the trait `name` gives it: `Arr<E>`, with
    /// `impl<D: Domain> Versioned<D, Int> for Arr<D>`, gives `[E, Int]`.
    /// The bound's args that `env` already fixes: `T` in `A: Acts<T, F>` once `xs` gave it.
    fn known_trait_args(&mut self, args: &[ast::Type], m: ModId, env: &GenEnv) -> Vec<Option<Ty>> {
        args.iter()
            .map(|a| {
                let fixed = self.impl_params(m, a).iter().all(|n| env.contains_key(n));
                fixed.then(|| {
                    let saved = (
                        std::mem::replace(&mut self.cur_mod, m),
                        std::mem::replace(&mut self.gen_env, env.clone()),
                    );
                    let ty = self.resolve_ty(a);
                    (self.cur_mod, self.gen_env) = saved;
                    ty
                })
            })
            .collect()
    }

    /// An impl's params its target leaves open, named only in its trait's
    /// args: `T` in `impl<T> Acts<T, T> for Plus`.
    pub(super) fn open_impl_params(&self, idx: usize, env: &GenEnv) -> Vec<String> {
        let m = self.impls[idx].mod_id;
        let mut out: Vec<String> = Vec::new();
        for t in &self.impls[idx].trait_args {
            for n in self.impl_params(m, t) {
                if n != "Self" && !env.contains_key(&n) && !out.contains(&n) {
                    out.push(n);
                }
            }
        }
        out
    }

    pub(super) fn impl_trait_args(
        &mut self,
        ty: &Ty,
        name: &str,
        known: &[Option<Ty>],
    ) -> Option<(Vec<Ty>, GenEnv)> {
        let Ty::Struct(sid) = *ty else {
            return None;
        };
        for idx in 0..self.impls.len() {
            if self.impls[idx].trait_name.as_deref() != Some(name) {
                continue;
            }
            let Some(mut env) = self.struct_impl_env(idx, sid) else {
                continue;
            };
            let (m, offset) = (self.impls[idx].mod_id, self.impls[idx].offset);
            let trait_args = self.impls[idx].trait_args.clone();
            let open = self.open_impl_params(idx, &env);
            for (t, k) in trait_args.iter().zip(known) {
                if let Some(k) = k {
                    self.unify_ast_ty(t, k, m, &open, &mut env);
                }
            }
            if open.iter().any(|n| !env.contains_key(n)) {
                return None;
            }
            let opened: GenEnv = open
                .iter()
                .map(|n| (format!("impl{idx}.{n}"), env[n].clone()))
                .collect();
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
                std::mem::replace(&mut self.gen_env, env),
            );
            let out = trait_args.iter().map(|t| self.resolve_ty(t)).collect();
            (self.cur_mod, self.offset, self.gen_env) = saved;
            return Some((out, opened));
        }
        None
    }

    /// `act.tag0()` on a `Plus` the enclosing fn bound as `A: Acts<T, F>`:
    /// the method of `impl<T> Acts<T, T> for Plus` at the `T` that bound solved.
    pub(super) fn open_impl_method(&mut self, sid: StructId, method: &str) -> Option<FnId> {
        for idx in self.structs[sid].open_impls.clone() {
            let imp = &self.impls[idx];
            let (m, offset, body, trait_name) =
                (imp.mod_id, imp.offset, imp.body, imp.trait_name.clone());
            let Some(f) = body.stmts.iter().find_map(|s| match &s.kind {
                ast::StmtKind::Fn(f) if f.name.text == method => Some(f),
                _ => None,
            }) else {
                continue;
            };
            let mut env = self.structs[sid].env.clone();
            let open = self.open_impl_params(idx, &env);
            let args: Option<Vec<GenArg>> = open
                .iter()
                .map(|n| self.gen_env.get(&format!("impl{idx}.{n}")).cloned())
                .collect();
            let Some(args) = args else { continue };
            let key = (sid, idx, method.to_string(), args.clone());
            if let Some(&id) = self.open_impl_fns.get(&key) {
                return Some(id);
            }
            env.extend(open.into_iter().zip(args));
            let fn_id = self.declare_fn_in(f, m, offset, Some(sid), env.clone());
            self.open_impl_fns.insert(key, fn_id);
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
                std::mem::replace(&mut self.gen_env, env),
                self.self_struct.replace(sid),
            );
            let first = self.impls[idx].trait_args.first().copied();
            let rhs = first.map(|t| self.resolve_ty(t));
            self.resolve_fn_sig_with(fn_id, trait_name.as_deref(), rhs.as_ref(), None);
            (self.cur_mod, self.offset, self.gen_env, self.self_struct) = saved;
            return Some(fn_id);
        }
        None
    }
}
