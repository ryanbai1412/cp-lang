//! Which impls a type has, `impl<D: Domain> Tr for D` among them: one impl
//! for every domain, its methods made for a domain the first time a call
//! reaches them through it.

use super::*;

impl<'a> Lowerer<'a> {
    /// The param a blanket `impl<D: Domain> Tr for D` is over; its bound makes
    /// `D` the impl's own even where the module has a `D`.
    pub(super) fn domain_blanket_param(&self, idx: usize) -> Option<String> {
        let imp = &self.impls[idx];
        let ast::TypeKind::Named { name, args } = &imp.target.kind else {
            return None;
        };
        if !args.is_empty() {
            return None;
        }
        let domain = |t: &ast::Type| matches!(&t.kind, ast::TypeKind::Named { name: tn, .. } if tn.text == "Domain");
        imp.bounds
            .iter()
            .any(|b| b.name.text == name.text && b.traits.iter().any(domain))
            .then(|| name.text.clone())
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
            .or_else(|| self.pattern_impl_method(ty, method))
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
            let (at, trait_name) = ((t.mod_id, t.offset), t.name.clone());
            let env = GenEnv::from([("Self".to_string(), GenArg::Ty(ty.clone()))]);
            let fn_id = self.prim_method(f, at, at, env, ty, (Some(trait_name), "shape"));
            return Some(fn_id);
        }
        None
    }

    /// Declares `f`, written at `def`, as the method `ty` gets from an impl
    /// or trait read at `at`, named `{trait}_{kind}_{method}_{n}`.
    pub(super) fn prim_method(
        &mut self,
        f: &'a ast::FnDecl,
        at: (ModId, u32),
        def: (ModId, u32),
        env: GenEnv,
        ty: &Ty,
        (trait_name, kind): (Option<String>, &str),
    ) -> FnId {
        let saved = (
            std::mem::replace(&mut self.cur_mod, at.0),
            std::mem::replace(&mut self.offset, at.1),
        );
        let fn_id = self.declare_fn_in(f, def.0, def.1, None, env);
        let suffix = self.prim_methods.len();
        self.sigs[fn_id].name = format!(
            "{}_{kind}_{}_{suffix}",
            trait_name.as_deref().unwrap_or("impl"),
            f.name.text
        );
        self.prim_self.insert(fn_id, ty.clone());
        self.prim_methods
            .push((ty.clone(), f.name.text.clone(), fn_id));
        self.resolve_fn_sig(fn_id, trait_name.as_deref());
        (self.cur_mod, self.offset) = saved;
        fn_id
    }

    /// `D.m(..)` with `D` a domain, or the cuts `V.cuts` of one: the method
    /// a blanket domain impl gives it.
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
            let generic = matches!(&self.impls[idx].target.kind, ast::TypeKind::Named { args, .. } if !args.is_empty());
            if (generic && !matches!(ty, Ty::Struct(_))) || self.impl_env(idx, ty).is_some() {
                return true;
            }
        }
        false
    }
}
