//! Which impls a type has, blanket `impl<S: Tr> Tr2 for S` among them: one
//! impl for every type meeting its bounds, its methods made for a type the
//! first time a call reaches them through it.

use super::*;

impl<'a> Lowerer<'a> {
    /// The param a blanket `impl<S: Tr> Tr2 for S` is over.
    pub(super) fn blanket_param(&self, idx: usize) -> Option<String> {
        let imp = &self.impls[idx];
        let ast::TypeKind::Named { name, args } = &imp.target.kind else {
            return None;
        };
        (args.is_empty()
            && !self.mod_syms[imp.mod_id].contains_key(&name.text)
            && imp.bounds.iter().any(|b| b.name.text == name.text))
        .then(|| name.text.clone())
    }

    /// Blanket impl `idx`'s env at `ty`: its param is `ty`, and the names its bounds leave
    /// open are bound from `ty`'s impls. `None` where `ty` does not meet a bound.
    pub(super) fn blanket_env(&mut self, idx: usize, ty: &Ty) -> Option<GenEnv> {
        let param = self.blanket_param(idx)?;
        let (m, bounds) = (self.impls[idx].mod_id, self.impls[idx].bounds);
        let position = matches!(ty, Ty::Pos(_) | Ty::Bound(_));
        for b in bounds.iter().filter(|b| b.name.text == param) {
            for tr in &b.traits {
                let ast::TypeKind::Named { name, .. } = &tr.kind else {
                    return None;
                };
                match name.text.as_str() {
                    "Domain" if !position => return None,
                    "Domain" | "Seq" | "Bidir" | "Sized" | "Positional" | "Keyed" | "Indexed"
                    | "IndexedMut" => {}
                    t if !self.implements(ty, t) => return None,
                    _ => {}
                }
            }
        }
        let arg = match ty {
            Ty::Pos(d) => GenArg::Domain(*d),
            _ => GenArg::Ty(ty.clone()),
        };
        let mut env = GenEnv::from([(param, arg)]);
        self.bind_impl_bounds(bounds, m, &mut env).then_some(env)
    }

    /// The method an impl gives a value of a non-struct type: a primitive's
    /// own impl, a domain position's blanket impl, `?T`'s or a fn type's.
    pub(super) fn value_impl_method(&mut self, ty: &Ty, method: &str) -> Option<FnId> {
        let prim = self
            .prim_methods
            .iter()
            .find(|(t, m, _)| t == ty && m == method);
        prim.map(|p| p.2)
            .or_else(|| self.optional_impl_method(ty, method))
            .or_else(|| self.head_impl_method(ty, method))
            .or_else(|| self.shape_default_method(ty, method))
    }

    /// R08.1.7: a tuple or record meets a trait by its shape, so a method the
    /// trait gives a body is made for it as for a type with an empty impl:
    /// `(3, 4).rev()` is `Reversible`'s.
    fn shape_default_method(&mut self, ty: &Ty, method: &str) -> Option<FnId> {
        if !matches!(ty, Ty::Tuple(_) | Ty::Record(_)) {
            return None;
        }
        for tid in 0..self.traits.len() {
            let t = &self.traits[tid];
            if t.alias_of.is_some() || !t.generics.is_empty() {
                continue;
            }
            let Some(f) = t.methods.iter().copied().find(|m| {
                m.name.text == method
                    && m.generics.is_empty()
                    && !matches!(m.body, ast::FnBody::None)
            }) else {
                continue;
            };
            if !self.meets_by_shape(ty, tid) {
                continue;
            }
            let t = &self.traits[tid];
            let (m, offset, trait_name) = (t.mod_id, t.offset, t.name.clone());
            let env = GenEnv::from([("Self".to_string(), GenArg::Ty(ty.clone()))]);
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
            );
            let fn_id = self.declare_fn_in(f, m, offset, None, env);
            let suffix = self.prim_methods.len();
            self.sigs[fn_id].name = format!("{trait_name}_shape_{method}_{suffix}");
            self.prim_self.insert(fn_id, ty.clone());
            self.prim_methods
                .push((ty.clone(), method.to_string(), fn_id));
            self.resolve_fn_sig(fn_id, Some(trait_name.as_str()));
            (self.cur_mod, self.offset) = saved;
            return Some(fn_id);
        }
        None
    }

    /// The body trait `impls[idx]` gives `method` when the impl writes none:
    /// `impl<D: Domain> Target<D> for D` takes `Target`'s `to`, with the
    /// trait's parameters bound by the impl's trait arguments.
    pub(super) fn impl_trait_default(
        &mut self,
        idx: usize,
        method: &str,
        ty: &Ty,
        mut env: GenEnv,
    ) -> Option<(&'a ast::FnDecl, ModId, u32, GenEnv)> {
        let name = self.impls[idx].trait_name.clone()?;
        let t = self.traits.iter().find(|t| t.name == name)?;
        let f = t.methods.iter().copied().find(|m| {
            m.name.text == method && m.generics.is_empty() && !matches!(m.body, ast::FnBody::None)
        })?;
        let (tm, toff) = (t.mod_id, t.offset);
        let generics: Vec<String> = t.generics.iter().map(|g| g.to_string()).collect();
        let (im, ioff) = (self.impls[idx].mod_id, self.impls[idx].offset);
        let args = self.impls[idx].trait_args.clone();
        let tys: Vec<Ty> = self.quietly_in(im, ioff, env.clone(), |l| {
            args.iter().map(|a| l.resolve_ty(a)).collect()
        });
        for (g, t) in generics.into_iter().zip(tys) {
            env.insert(g, GenArg::Ty(t));
        }
        env.insert("Self".to_string(), GenArg::Ty(ty.clone()));
        Some((f, tm, toff, env))
    }

    /// Whether `ty` has an `impl` of the library trait `name`. A name that is not a trait declared
    /// in a module (a structural one), or an `impl` over a generic type, is taken as met.
    pub(super) fn implements(&mut self, ty: &Ty, name: &str) -> bool {
        let Some(Sym::Trait(tid)) = self.get_sym(name) else {
            return true;
        };
        if let Some(supers) = self.traits[tid].alias_of.clone() {
            return supers.iter().all(|s| self.implements(ty, s));
        }
        for idx in 0..self.impls.len() {
            if self.impls[idx].trait_name.as_deref() != Some(name) {
                continue;
            }
            let (m, offset, target) = (
                self.impls[idx].mod_id,
                self.impls[idx].offset,
                self.impls[idx].target,
            );
            if matches!(&target.kind, ast::TypeKind::Named { args, .. } if !args.is_empty()) {
                return true;
            }
            if self.blanket_param(idx).is_some() {
                if self.blanket_env(idx, ty).is_some() {
                    return true;
                }
                continue;
            }
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
            );
            let got = self.resolve_ty(target);
            (self.cur_mod, self.offset) = saved;
            if got == *ty || got == Ty::Error {
                return true;
            }
        }
        false
    }
}
