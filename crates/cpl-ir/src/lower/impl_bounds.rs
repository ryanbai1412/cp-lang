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
                        if let Some(given) = self.impl_trait_args(&seq, trait_name) {
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

    /// The args `ty`'s impl of the trait `name` gives it: `Arr<E>`, with
    /// `impl<D: Domain> Versioned<D, Int> for Arr<D>`, gives `[E, Int]`.
    pub(super) fn impl_trait_args(&mut self, ty: &Ty, name: &str) -> Option<Vec<Ty>> {
        let Ty::Struct(sid) = *ty else {
            return None;
        };
        let origin = self.structs[sid].origin_gen.clone();
        for idx in 0..self.impls.len() {
            if self.impls[idx].trait_name.as_deref() != Some(name) {
                continue;
            }
            let (m, offset, target) = (
                self.impls[idx].mod_id,
                self.impls[idx].offset,
                self.impls[idx].target,
            );
            let ast::TypeKind::Named {
                name: head,
                args: targs,
            } = &target.kind
            else {
                continue;
            };
            let fits = match self.mod_syms[m].get(&head.text).copied() {
                Some(Sym::Struct(id)) => id == sid,
                Some(Sym::GenStruct(g)) => origin.as_ref().is_some_and(|(o, _)| *o == g),
                _ => false,
            };
            if !fits {
                continue;
            }
            let open = self.impl_params(m, target);
            let mut env = self.structs[sid].env.clone();
            let given = origin.as_ref().map_or(&[][..], |(_, a)| &a[..]);
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
            let trait_args = self.impls[idx].trait_args.clone();
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
                std::mem::replace(&mut self.gen_env, env),
            );
            let out = trait_args.iter().map(|t| self.resolve_ty(t)).collect();
            (self.cur_mod, self.offset, self.gen_env) = saved;
            return Some(out);
        }
        None
    }
}
