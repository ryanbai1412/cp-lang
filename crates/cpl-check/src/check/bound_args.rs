//! The arguments a bound gives its trait: inside `fn f<S: Indexed<E, R>>`,
//! `S`'s members have the trait's `D` and `T` as `E` and `R`.

use super::*;
use cpl_syntax::ast;

/// `(param, trait, args)` of each bound that gives its trait arguments.
pub(crate) type BoundArgs = Vec<(String, String, Vec<Ty>)>;

impl Checker<'_> {
    /// The bindings of an impl's generics for `ty`: its target's, then each generic only a
    /// head bound names, from the bound parameter's own impl of that bound.
    pub(crate) fn impl_bindings(
        &mut self,
        target: &Ty,
        head_bounds: &BoundArgs,
        ty: &Ty,
    ) -> crate::ty::Bindings {
        let mut on = crate::ty::Bindings::default();
        target.unify(ty, &mut on);
        for (param, trait_name, args) in head_bounds {
            let Some(actual) = on.get(param).cloned() else {
                continue;
            };
            if let Some(given) = self.trait_args_of(&actual, trait_name, args.len()) {
                for (want, got) in args.iter().zip(&given) {
                    want.unify(got, &mut on);
                }
            }
        }
        on
    }

    /// `f`'s bounds: the record rows (`R: (u: V, ..)`) and the traits (`T: Ord`) its type
    /// parameters must meet.
    pub(crate) fn fn_bounds(
        &mut self,
        f: &ast::FnDecl,
    ) -> (crate::def::RowBounds, Vec<(String, String)>, BoundArgs) {
        let outer = self.generics.len();
        self.generics
            .extend(f.generics.iter().map(|g| g.text.clone()));
        let mut rows = Vec::new();
        let mut traits = Vec::new();
        let mut bound_args = Vec::new();
        for b in &f.bounds {
            for t in &b.traits {
                match &t.kind {
                    ast::TypeKind::Row(fields) => {
                        let resolved = fields
                            .iter()
                            .map(|(n, ty)| (n.text.clone(), self.resolve_type(ty)))
                            .collect();
                        rows.push((b.name.text.clone(), resolved));
                    }
                    ast::TypeKind::Named { name: tname, args } => {
                        traits.push((b.name.text.clone(), tname.text.clone()));
                        if !args.is_empty() {
                            let args = args.iter().map(|a| self.resolve_type(a)).collect();
                            bound_args.push((b.name.text.clone(), tname.text.clone(), args));
                        }
                    }
                    _ => {}
                }
            }
        }
        self.generics.truncate(outer);
        (rows, traits, bound_args)
    }

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

    /// `last<S: Bidir, T>(w)`: a trait parameter the fn leaves free is the
    /// one the argument's impl of the bound gives it, by name.
    pub(crate) fn bind_bound_params(
        &mut self,
        generics: &[String],
        bounds: &[(String, Vec<String>)],
        bindings: &mut crate::ty::Bindings,
    ) {
        for (param, traits) in bounds {
            let Some(Ty::Named { name, args }) = bindings.get(param).map(|t| self.settle(t)) else {
                continue;
            };
            let on_ty = Ty::Named {
                name: name.clone(),
                args,
            };
            for trait_name in traits {
                let Some(Def::Trait {
                    generics: of_trait, ..
                }) = self.type_def(trait_name).as_deref().cloned()
                else {
                    continue;
                };
                let free = |g: &String, b: &crate::ty::Bindings| {
                    generics.contains(g) && !b.contains_key(g)
                };
                if !of_trait.iter().any(|g| free(g, bindings)) {
                    continue;
                }
                let found = self.env.impls(&name).into_iter().find_map(|(t, def)| {
                    let named =
                        t.as_deref().and_then(|t| t.split('<').next()) == Some(trait_name.as_str());
                    match &*def {
                        Def::Impl {
                            trait_args,
                            target,
                            head_bounds,
                            ..
                        } if named => {
                            Some((trait_args.clone(), target.clone(), head_bounds.clone()))
                        }
                        _ => None,
                    }
                });
                let Some((trait_args, target, head_bounds)) = found else {
                    continue;
                };
                let on = self.impl_bindings(&target, &head_bounds, &on_ty);
                for (g, arg) in of_trait.iter().zip(trait_args) {
                    if free(g, bindings) {
                        bindings.insert(g.clone(), arg.subst(&on));
                    }
                }
            }
        }
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

    /// At a call: each generic a bound's trait arguments name, from the
    /// type the bound's parameter is bound to, so `sum(xs)` over
    /// `S: Seq<T>` given a `List<Int>` has `T = Int`.
    pub(crate) fn bind_bound_args(&mut self, bindings: &mut crate::ty::Bindings) {
        let Some(bound_args) = self.call_bound_args.clone() else {
            return;
        };
        for (param, trait_name, args) in &bound_args {
            let Some(actual) = bindings.get(param).cloned() else {
                continue;
            };
            let actual = self.settle(&actual);
            if let Some(given) = self.trait_args_of(&actual, trait_name, args.len()) {
                // `impl<T> Acts<T, T> for Plus`: the impl's own `T` is the
                // bound's, where the call already fixed it.
                let mut open = crate::ty::Bindings::default();
                for (want, got) in args.iter().zip(&given) {
                    let known = want.subst(bindings);
                    if !known.mentions_param() {
                        got.unify(&known, &mut open);
                    }
                }
                for (want, got) in args.iter().zip(&given) {
                    want.unify(&got.subst(&open), bindings);
                }
            }
        }
    }

    /// The arguments `ty` gives trait `trait_name`: a bound's in a generic
    /// body, an `impl`'s as written, or a built-in sequence's key and
    /// element.
    pub(crate) fn trait_args_of(&mut self, ty: &Ty, trait_name: &str, n: usize) -> Option<Vec<Ty>> {
        if let Ty::Param(p) = ty {
            return (self.param_bound_args.iter().rev())
                .find(|(q, t, args)| q == p && t == trait_name && args.len() == n)
                .map(|(_, _, args)| args.clone());
        }
        if let Some(args) = self.impl_trait_args(ty, trait_name, n) {
            return Some(args);
        }
        if n == 0
            || !self.trait_implies(trait_name, "Seq")
            || !self.implements_trait(ty, trait_name)
        {
            return None;
        }
        let elem = self.element_ty(ty);
        if elem.has_vague() {
            return None;
        }
        let key = match ty {
            Ty::Array { index, .. } if index.len() == 1 => index[0].clone(),
            Ty::Str => Ty::list_index(),
            Ty::Named { name, .. } if name == "List" => Ty::list_index(),
            _ => Ty::Unknown,
        };
        let mut out = vec![Ty::Unknown; n - 1];
        if let Some(last) = out.last_mut() {
            *last = key;
        }
        out.push(elem);
        Some(out)
    }
    /// The arguments an `impl` of `trait_name` for struct `ty` gives the trait, bound
    /// through the impl's head: `impl Seq<(D, T)> for Items<S: Keyed<D, T>>`.
    pub(crate) fn impl_trait_args(
        &mut self,
        ty: &Ty,
        trait_name: &str,
        n: usize,
    ) -> Option<Vec<Ty>> {
        if let Ty::Named { name, .. } = ty {
            let written = self.global(&crate::def::impl_name(Some(trait_name), name));
            let listed = (self.env.impls(name).into_iter())
                .filter(|(t, _)| t.as_deref().and_then(|t| t.split('<').next()) == Some(trait_name))
                .map(|(_, d)| d);
            for def in written.into_iter().chain(listed) {
                if let Def::Impl {
                    trait_args,
                    target,
                    head_bounds,
                    ..
                } = &*def
                {
                    if trait_args.len() == n {
                        let on = self.impl_bindings(target, head_bounds, ty);
                        return Some(trait_args.iter().map(|t| t.subst(&on)).collect());
                    }
                }
            }
        }
        None
    }
}

/// `of: T -> U = x => x`: each generic a defaulted identity lambda gives
/// back, with the parameter type it then is (`U` is `T`).
pub(crate) fn identity_defaults(f: &ast::FnDecl, ty: &Ty) -> Vec<(String, Ty)> {
    let Ty::Fn { params, .. } = ty else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for p in &f.params {
        let Some(ast::Expr {
            kind: ast::ExprKind::Lambda {
                params: lp, body, ..
            },
            ..
        }) = &p.default
        else {
            continue;
        };
        let identity = matches!((lp.as_slice(), &body.kind),
            ([one], ast::ExprKind::Name(b)) if matches!(&one.kind, ast::PatternKind::Name(x) if x == b));
        let sig = params.iter().find(|q| q.name == p.name.text).map(|q| &q.ty);
        if let (true, Some(Ty::Fn { params: takes, ret })) = (identity, sig) {
            if let ([taken], Ty::Param(r)) = (takes.as_slice(), ret.as_ref()) {
                out.push((r.clone(), taken.ty.clone()));
            }
        }
    }
    out
}
