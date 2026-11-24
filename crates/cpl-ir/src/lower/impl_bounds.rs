//! An impl head's inline bounds (R12.3): `impl Seq<T> for Wrap<S: Seq<T>>`
//! takes `T` from what `S` is, and gives `Wrap<S>` nothing where `S` is not a
//! `Seq`; a bound on any trait, `S: Versioned<D, T>`, binds `D` and `T` from
//! `S`'s impl of it.

use super::signature::ranged_name;
use super::*;

/// A trait's args read off one impl, with the impl's params bound.
type TraitArgs = (Vec<Ty>, GenEnv);

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
        self.bind_bounds_open(bounds, m, &[], env)
    }

    /// [`Self::bind_impl_bounds`] where `declared` names the generics a fn
    /// declares, open even when they shadow a module name such as a domain `V`.
    pub(super) fn bind_bounds_open(
        &mut self,
        bounds: &[ast::Bound],
        m: ModId,
        declared: &[String],
        env: &mut GenEnv,
    ) -> bool {
        let mut open: Vec<String> = declared
            .iter()
            .filter(|n| !env.contains_key(*n))
            .cloned()
            .collect();
        for tr in bounds.iter().flat_map(|b| &b.traits) {
            for name in self.impl_params(m, tr) {
                // `M.Fold` is fixed once `M` is, never open itself.
                let head = name.split_once('.').map_or(&*name, |(o, _)| o);
                if !env.contains_key(head) && !open.contains(&name) {
                    open.push(name);
                }
            }
        }
        for b in bounds {
            let seq = match env.get(&b.name.text) {
                Some(GenArg::Ty(t)) => t.clone(),
                Some(GenArg::Domain(d)) => Ty::Pos(*d),
                _ => continue,
            };
            for tr in &b.traits {
                let ast::TypeKind::Named { name, args } = &tr.kind else {
                    continue;
                };
                let trait_name = name.text.as_str();
                let is_struct = matches!(seq, Ty::Struct(_));
                if is_struct && !self.implements(&seq, trait_name) {
                    return false;
                }
                let roles = (self.seq_roles(trait_name).map(|r| (r.key, r.elem)))
                    .or((trait_name == "Sized").then_some((None, None)))
                    .filter(|_| !is_struct);
                let (key, elem) = match roles {
                    Some((k, e)) => (k.and_then(|i| args.get(i)), e.and_then(|i| args.get(i))),
                    None => {
                        let known = self.known_trait_args(args, m, &open, env);
                        if let Some((given, opened)) =
                            self.impl_trait_args(&seq, trait_name, &known, Some((args, &*env)))
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
                if !builtin {
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

    /// The generic env impl `idx` binds when it is for `ty`: its struct's
    /// args, the domain of a blanket domain impl, or what a shape pattern
    /// (`[V]S`, `(T, T) -> T`, a row record) binds.
    pub(super) fn impl_env(&mut self, idx: usize, ty: &Ty) -> Option<GenEnv> {
        if let Ty::Struct(sid) = *ty {
            return self.struct_impl_env(idx, sid);
        }
        let (m, offset, target) = (
            self.impls[idx].mod_id,
            self.impls[idx].offset,
            self.impls[idx].target,
        );
        if let ast::TypeKind::Named { name, .. } = &target.kind {
            if let Some(Sym::Struct(_) | Sym::GenStruct(_)) = self.mod_syms[m].get(&name.text) {
                return None;
            }
        }
        if let Some(param) = self.domain_blanket_param(idx) {
            return match ty {
                Ty::Pos(d) => Some(GenEnv::from([(param, GenArg::Domain(*d))])),
                Ty::Bound(_) => Some(GenEnv::from([(param, GenArg::Ty(ty.clone()))])),
                Ty::Tuple(ts) if ts.iter().all(|t| matches!(t, Ty::Pos(_))) => {
                    Some(GenEnv::from([(param, GenArg::Ty(ty.clone()))]))
                }
                _ => None,
            };
        }
        let env = if self.is_pattern_impl(idx) {
            self.pattern_impl_env(idx, ty)?
        } else {
            GenEnv::default()
        };
        let (got, env) = self.quietly_in(m, offset, env, |l| {
            (l.resolve_ty(target), std::mem::take(&mut l.gen_env))
        });
        (got == *ty).then_some(env)
    }

    /// The args `ty`'s impl of the trait `name` gives it: `Arr<E>`, with
    /// `impl<D: Domain> Versioned<D, Int> for Arr<D>`, gives `[E, Int]`.
    /// The bound's args that `env` already fixes: `T` in `A: Acts<T, F>` once `xs` gave it.
    fn known_trait_args(
        &mut self,
        args: &[ast::Type],
        m: ModId,
        open: &[String],
        env: &GenEnv,
    ) -> Vec<Option<Ty>> {
        args.iter()
            .map(|a| {
                // `M.Fold` is fixed once `M` is.
                let fixed = (self.impl_params(m, a).iter())
                    .all(|n| env.contains_key(n.split('.').next().unwrap_or(n)))
                    && !open.iter().any(|n| mono::ty_mentions(a, n));
                fixed.then(|| self.quietly_in(m, self.offset, env.clone(), |l| l.resolve_ty(a)))
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
        ranged: Option<(&[ast::Type], &GenEnv)>,
    ) -> Option<(Vec<Ty>, GenEnv)> {
        for idx in 0..self.impls.len() {
            if self.impls[idx].trait_name.as_deref() != Some(name) {
                continue;
            }
            let key = (idx, ty.clone());
            if self.growth.impl_args.contains(&key) {
                continue;
            }
            self.growth.impl_args.push(key);
            let got = self.impl_trait_args_at(idx, ty, known, ranged);
            self.growth.impl_args.pop();
            if let Some(got) = got {
                return got;
            }
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

    /// [`Self::impl_trait_args`] through impl `idx`: `None` when it is not for `ty`.
    fn impl_trait_args_at(
        &mut self,
        idx: usize,
        ty: &Ty,
        known: &[Option<Ty>],
        ranged: Option<(&[ast::Type], &GenEnv)>,
    ) -> Option<Option<TraitArgs>> {
        let mut env = self.impl_env(idx, ty)?;
        let (m, offset) = (self.impls[idx].mod_id, self.impls[idx].offset);
        if !self.bind_impl_bounds(self.impls[idx].bounds, m, &mut env) {
            return None;
        }
        let trait_args = self.impls[idx].trait_args.clone();
        let open = self.open_impl_params(idx, &env);
        for (i, (t, k)) in trait_args.iter().zip(known).enumerate() {
            if let Some(k) = k {
                self.unify_ast_ty(t, k, m, &open, &mut env);
            }
            // `A: Acts<T, F>` with `T = -10..=10`: the impl's `T` is that range.
            let given = ranged.and_then(|(args, outer)| ranged_name(outer, args.get(i)?));
            if let (ast::TypeKind::Named { name, .. }, Some(g)) = (&t.kind, given) {
                if open.contains(&name.text) {
                    env.insert(name.text.clone(), g);
                }
            }
        }
        if open.iter().any(|n| !env.contains_key(n)) {
            return Some(None);
        }
        let opened: GenEnv = open
            .iter()
            .map(|n| (format!("impl{idx}.{n}"), env[n].clone()))
            .collect();
        let out = self.quietly_in(m, offset, env, |l| {
            trait_args.iter().map(|t| l.resolve_ty(t)).collect()
        });
        Some(Some((out, opened)))
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
