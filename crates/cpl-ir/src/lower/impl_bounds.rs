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
                let roles = (self.seq_roles(trait_name).map(|r| (r.key, r.elem)))
                    .or((trait_name == "Sized").then_some((None, None)));
                let (key, elem) = match roles {
                    Some((k, e)) => (k.and_then(|i| args.get(i)), e.and_then(|i| args.get(i))),
                    None => {
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
                    self.unify_ast_ty(k, &seq.key_ty(), m, &open, env);
                }
                if let (Some(t), Some(e)) = (elem, seq.seq_elem()) {
                    self.unify_ast_ty(t, &e, m, &open, env);
                }
            }
        }
        true
    }

    /// The generic env impl `idx` binds when its head names the struct `sid`:
    /// `impl Next<V> for Graph<V, A>` on `Graph<X, Y>` binds `V = X`, `A = Y`.
    pub(super) fn struct_impl_env(&self, idx: usize, sid: StructId) -> Option<GenEnv> {
        let (m, target) = (self.impls[idx].mod_id, self.impls[idx].target);
        if let ast::TypeKind::ItemOf(of) = &target.kind {
            let ident = of.fn_item_ident()?;
            let fits = self
                .item_types
                .iter()
                .any(|(k, s)| *s == sid && *k == ident);
            return fits.then(|| self.structs[sid].env.clone());
        }
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

    /// `impl<T> Op<T> for E3` at an `E3` the enclosing fn bound as `M: Op<E>`: the impl's env
    /// with the `T` that bound solved, which names only the trait argument.
    pub(super) fn opened_impl_env(&self, idx: usize, sid: StructId) -> Option<GenEnv> {
        let mut env = self.struct_impl_env(idx, sid)?;
        let open = self.open_impl_params(idx, &env);
        if open.is_empty() {
            return None;
        }
        for n in open {
            let arg = self.gen_env.get(&format!("impl{idx}.{n}"))?.clone();
            env.insert(n, arg);
        }
        Some(env)
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
            let key = mono::InstKey::OpenImpl(sid, idx, method.to_string(), args.clone());
            if let Some(&id) = self.insts.get(&key) {
                return Some(id);
            }
            env.extend(open.into_iter().zip(args));
            let fn_id = self.declare_fn_in(f, m, offset, Some(sid), env.clone());
            self.insts.insert(key, fn_id);
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
