//! Which impls a type has, `impl<D: Domain> Tr for D` among them: one impl
//! for every domain, its methods made for a domain the first time a call
//! reaches them through it.

use super::*;

impl Lowerer<'_> {
    /// The param a blanket `impl<D: Domain> Tr for D` is over.
    pub(super) fn domain_blanket_param(&self, idx: usize) -> Option<String> {
        let imp = &self.impls[idx];
        let ast::TypeKind::Named { name, args } = &imp.target.kind else {
            return None;
        };
        if !args.is_empty() || self.mod_syms[imp.mod_id].contains_key(&name.text) {
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
        if let Ty::Pos(_) = ty {
            return self.domain_impl_method(ty.clone(), method);
        }
        let prim = self
            .prim_methods
            .iter()
            .find(|(t, m, _)| t == ty && m == method);
        prim.map(|p| p.2)
            .or_else(|| self.optional_impl_method(ty, method))
            .or_else(|| self.fn_impl_method(ty, method))
    }

    /// `D.m(..)` with `D` a domain, or the cuts `V.cuts` of one: the method
    /// a blanket domain impl gives it.
    pub(super) fn domain_impl_method(&mut self, ty: Ty, method: &str) -> Option<FnId> {
        let arg = match ty {
            Ty::Pos(d) => GenArg::Domain(d),
            Ty::Bound(_) => GenArg::Ty(ty.clone()),
            _ => return None,
        };
        if let Some(found) = self
            .prim_methods
            .iter()
            .find(|(t, m, _)| *t == ty && m == method)
        {
            return Some(found.2);
        }
        for idx in 0..self.impls.len() {
            let Some(param) = self.domain_blanket_param(idx) else {
                continue;
            };
            let (m, offset, body) = (
                self.impls[idx].mod_id,
                self.impls[idx].offset,
                self.impls[idx].body,
            );
            let Some(f) = body.stmts.iter().find_map(|item| match &item.kind {
                ast::StmtKind::Fn(f) if f.name.text == method && f.generics.is_empty() => Some(f),
                _ => None,
            }) else {
                continue;
            };
            let trait_name = self.impls[idx].trait_name.clone();
            let env = GenEnv::from([(param, arg.clone())]);
            let saved = (
                std::mem::replace(&mut self.cur_mod, m),
                std::mem::replace(&mut self.offset, offset),
            );
            let fn_id = self.declare_fn_in(f, m, offset, None, env);
            let suffix = self.prim_methods.len();
            self.sigs[fn_id].name = format!(
                "{}_dom_{}_{suffix}",
                trait_name.as_deref().unwrap_or("impl"),
                f.name.text
            );
            self.prim_self.insert(fn_id, ty.clone());
            self.prim_methods
                .push((ty.clone(), method.to_string(), fn_id));
            self.resolve_fn_sig(fn_id, trait_name.as_deref());
            (self.cur_mod, self.offset) = saved;
            return Some(fn_id);
        }
        None
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
                match ty {
                    Ty::Struct(sid) => {
                        if self.impl_env_for(idx, *sid).is_some() {
                            return true;
                        }
                        continue;
                    }
                    _ => return true,
                }
            }
            if self.domain_blanket_param(idx).is_some() {
                if matches!(ty, Ty::Pos(_) | Ty::Bound(_)) {
                    return true;
                }
                continue;
            }
            // `impl Bidir<T> for [D]T` covers built-in shapes, never a struct.
            if !self.impl_params(m, target).is_empty() {
                if !matches!(ty, Ty::Struct(_)) {
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
