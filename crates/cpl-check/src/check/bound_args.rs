//! The arguments a bound gives its trait: inside `fn f<S: Indexed<E, R>>`,
//! `S`'s members have the trait's `D` and `T` as `E` and `R`.

use super::*;
use cpl_syntax::ast;

impl Checker<'_> {
    /// Puts a fn's bounds in scope for its body; [`Self::leave_bounds`] takes them out.
    pub(crate) fn enter_fn_bounds(&mut self, f: &ast::FnDecl) -> (usize, usize) {
        let outer = (self.param_bounds.len(), self.param_bound_args.len());
        self.param_bounds.extend(super::domain_use::body_bounds(f));
        self.bounds_exist(&f.bounds);
        let bound_args = self.quietly(|c| c.bound_args(&f.bounds));
        self.param_bound_args.extend(bound_args);
        outer
    }

    pub(crate) fn leave_bounds(&mut self, (bounds, args): (usize, usize)) {
        self.param_bounds.truncate(bounds);
        self.param_bound_args.truncate(args);
    }

    /// The `(param, trait, args)` of each bound that gives its trait arguments.
    pub(crate) fn bound_args(&mut self, bounds: &[ast::Bound]) -> Vec<(String, String, Vec<Ty>)> {
        let mut out = Vec::new();
        for b in bounds {
            for t in &b.traits {
                if let ast::TypeKind::Named { name, args } = &t.kind {
                    if !args.is_empty() {
                        let args = args.iter().map(|a| self.resolve_type(a)).collect();
                        out.push((b.name.text.clone(), name.text.clone(), args));
                    }
                }
            }
        }
        out
    }

    /// `Self` and the trait's own parameters, as bound `param: trait_name<..>` gives them.
    pub(crate) fn bound_bindings(&mut self, param: &str, trait_name: &str) -> crate::ty::Bindings {
        let mut on = crate::ty::Bindings::default();
        on.insert("Self".into(), Ty::Param(param.into()));
        let args = self
            .param_bound_args
            .iter()
            .rev()
            .find(|(p, t, _)| p == param && t == trait_name)
            .map(|(_, _, args)| args.clone());
        if let (Some(args), Some(Def::Trait { generics, .. })) =
            (args, self.type_def(trait_name).as_deref())
        {
            for (g, a) in generics.iter().zip(args) {
                on.insert(g.clone(), a);
            }
        }
        on
    }

    /// The type of method `name` that one of `param`'s bounds gives it.
    pub(crate) fn bound_method(&mut self, param: &str, name: &str) -> Option<Ty> {
        let bounds: Vec<String> = self
            .param_bounds
            .iter()
            .filter(|(p, _)| p == param)
            .flat_map(|(_, ts)| ts.iter().cloned())
            .collect();
        for trait_name in bounds {
            let ty = match self.type_def(&trait_name).as_deref() {
                Some(Def::Trait { methods, .. }) => methods
                    .iter()
                    .find(|m| m.name == name)
                    .map(|m| m.ty.clone()),
                _ => None,
            };
            if let Some(ty) = ty {
                return Some(ty.subst(&self.bound_bindings(param, &trait_name)));
            }
        }
        None
    }
}
