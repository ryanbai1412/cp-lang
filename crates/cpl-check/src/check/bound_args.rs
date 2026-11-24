//! The arguments a bound gives its trait: inside `fn f<S: Indexed<E, R>>`,
//! `S`'s members have the trait's `D` and `T` as `E` and `R`.

use super::*;
use cpl_syntax::ast;

/// `(param, trait, args)` of each bound that gives its trait arguments.
pub(crate) type BoundArgs = Vec<(String, String, Vec<Ty>)>;

impl Checker<'_> {
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
                    ast::TypeKind::Named { name: tname, .. } => {
                        traits.push((b.name.text.clone(), tname.text.clone()));
                    }
                    _ => {}
                }
            }
        }
        let bound_args = self.bound_args(&f.bounds);
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
    pub(crate) fn bound_args(&mut self, bounds: &[ast::Bound]) -> crate::def::BoundArgs {
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

    /// An `impl`'s trait arguments and its generics' bound arguments, as written.
    pub(crate) fn impl_head_args(
        &mut self,
        trait_name: Option<&ast::Type>,
        bounds: &[ast::Bound],
    ) -> (Vec<Ty>, crate::def::BoundArgs) {
        let trait_args = match trait_name.map(|t| &t.kind) {
            Some(ast::TypeKind::Named { args, .. }) => args
                .iter()
                .map(|a| self.quietly(|c| c.resolve_type(a)))
                .collect(),
            _ => Vec::new(),
        };
        (trait_args, self.quietly(|c| c.bound_args(bounds)))
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
                let Some(args) = self.trait_args_of(&on_ty, trait_name, of_trait.len()) else {
                    continue;
                };
                for (g, arg) in of_trait.iter().zip(args) {
                    if free(g, bindings) {
                        bindings.insert(g.clone(), arg);
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

    /// A bare op (`+`, `max`) passed for `M: Op<E>` is the `Monoid<E>` the
    /// trait has an impl for, once `E` is known, at a generic bound as at a
    /// concrete one; a closure keeps its fn type (R08.1.9, decision 456).
    pub(crate) fn bare_ops_take_monoid(
        &mut self,
        bound_args: &[(String, String, Vec<Ty>)],
        params: &[crate::ty::Param],
        args: &[ast::Arg],
        bindings: &mut crate::ty::Bindings,
    ) {
        let positional: Vec<&crate::ty::Param> = params.iter().filter(|p| !p.named_only).collect();
        let mut next = 0;
        for arg in args {
            let param = match &arg.name {
                Some(n) => params.iter().find(|p| p.name == n.text),
                None => {
                    next += 1;
                    positional.get(next - 1).copied()
                }
            };
            let Some(Ty::Param(m)) = param.map(|p| &p.ty) else {
                continue;
            };
            if !self.is_bare_op(&arg.value) {
                continue;
            }
            for (p, trait_name, targs) in bound_args {
                if p != m {
                    continue;
                }
                let want: Vec<Ty> = targs
                    .iter()
                    .map(|t| self.settle(&t.subst(bindings)))
                    .collect();
                if let Some(ty) = self.monoid_impl_for(trait_name, &want) {
                    bindings.insert(m.clone(), ty);
                }
            }
        }
    }

    /// At a call: a fn item passed for a parameter `M: Tr` whose own type
    /// `type_of(f)` has an impl of `Tr` binds `M` to that type, before its
    /// value would coerce to its fn type (R08.1.9).
    pub(crate) fn fn_items_take_their_type(
        &mut self,
        def: &Def,
        params: &[crate::ty::Param],
        args: &[ast::Arg],
        bindings: &mut crate::ty::Bindings,
    ) {
        let Def::Fn { bounds, .. } = def else {
            return;
        };
        let positional: Vec<&crate::ty::Param> = params.iter().filter(|p| !p.named_only).collect();
        let mut next = 0;
        for arg in args {
            let param = match &arg.name {
                Some(n) => params.iter().find(|p| p.name == n.text),
                None => {
                    next += 1;
                    positional.get(next - 1).copied()
                }
            };
            let Some(Ty::Param(m)) = param.map(|p| &p.ty) else {
                continue;
            };
            let Some(item) = arg.value.fn_item_type() else {
                continue;
            };
            if let ast::ExprKind::Name(n) = &arg.value.kind {
                if self.local(n).is_some()
                    || !matches!(self.global(n).as_deref(), Some(Def::Fn { .. }))
                {
                    continue;
                }
            }
            let Some((_, traits)) = bounds.iter().find(|(p, _)| p == m) else {
                continue;
            };
            let has_impl = self.env.impls(&item).iter().any(|(t, _)| {
                t.as_deref()
                    .and_then(|t| t.split('<').next())
                    .is_some_and(|t| traits.iter().any(|w| w == t))
            });
            if has_impl {
                bindings.insert(
                    m.clone(),
                    Ty::Named {
                        name: item,
                        args: Vec::new(),
                    },
                );
            }
        }
    }

    /// The struct an impl of `trait_name<want..>` is for whose `@from_op` method takes a bare
    /// op, `Monoid<..>` in std.
    fn monoid_impl_for(&mut self, trait_name: &str, want: &[Ty]) -> Option<Ty> {
        let mut impls = Vec::new();
        for target in self.env.impl_targets() {
            if let Some(Def::Struct {
                from_op: Some(_), ..
            }) = self.type_def(&target).as_deref()
            {
                impls.extend(self.env.impls(&target));
            }
        }
        for (t, def) in impls {
            if t.as_deref().and_then(|t| t.split('<').next()) != Some(trait_name) {
                continue;
            }
            let Def::Impl {
                trait_args, target, ..
            } = &*def
            else {
                continue;
            };
            if trait_args.len() != want.len() {
                continue;
            }
            let mut on = crate::ty::Bindings::default();
            for (a, w) in trait_args.iter().zip(want) {
                a.unify(w, &mut on);
            }
            let ty = target.subst(&on);
            if !ty.has_vague() && !ty.mentions_param() {
                return Some(ty);
            }
        }
        None
    }

    /// `+` or an unshadowed `max`/`min`/`gcd` written bare, which `Monoid.of` gives an identity for.
    fn is_bare_op(&self, e: &ast::Expr) -> bool {
        match &e.kind {
            ast::ExprKind::Paren(inner) => self.is_bare_op(inner),
            ast::ExprKind::Name(n) => {
                matches!(n.as_str(), "max" | "min" | "gcd") && self.local(n).is_none()
            }
            ast::ExprKind::Lambda { body, .. } => {
                matches!(&body.kind, ast::ExprKind::Binary { op_span, .. } if *op_span == e.span)
            }
            _ => false,
        }
    }

    /// At a call: each generic a bound's trait arguments name, from the
    /// type the bound's parameter is bound to, so `sum(xs)` over
    /// `S: Seq<T>` given a `List<Int>` has `T = Int`.
    pub(crate) fn bind_bound_args(&mut self, bindings: &mut crate::ty::Bindings) {
        let Some(bound_args) = self.call_bound_args.take() else {
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
        self.call_bound_args = Some(bound_args);
    }

    /// `fn f<A: Keyed<K, _>, B: Keyed<K, _>, K>`: a type parameter named in several bounds is
    /// one type, so every bound's argument must give the same `K` (decision 543).
    pub(crate) fn shared_bound_args_agree(
        &mut self,
        fn_name: &str,
        bound_args: &BoundArgs,
        bindings: &crate::ty::Bindings,
        span: Span,
    ) {
        let mut seen: Vec<(String, usize, Ty)> = Vec::new();
        for (i, (param, trait_name, args)) in bound_args.iter().enumerate() {
            let Some(actual) = bindings.get(param) else {
                continue;
            };
            let actual = self.settle(actual);
            let Some(given) = self.trait_args_of(&actual, trait_name, args.len()) else {
                continue;
            };
            for (want, got) in args.iter().zip(&given) {
                let Ty::Param(g) = want else {
                    continue;
                };
                if got.mentions_param() || got.has_vague() {
                    continue;
                }
                let Some((_, j, first)) = seen.iter().find(|(h, ..)| h == g).cloned() else {
                    seen.push((g.clone(), i, got.clone()));
                    continue;
                };
                if got.fits(&first) || first.fits(got) {
                    continue;
                }
                let shown = |(p, t, a): &(String, String, Vec<Ty>)| {
                    let a: Vec<String> = a.iter().map(|t| t.to_string()).collect();
                    format!("{p}: {t}<{}>", a.join(", "))
                };
                let (fa, fb) = (&bound_args[j].0, param);
                self.report(
                    span,
                    msgs::check::SHARED_BOUND_ARG,
                    &[
                        &fn_name,
                        g,
                        &shown(&bound_args[j]),
                        &shown(&bound_args[i]),
                        fa,
                        &first,
                        fb,
                        got,
                    ],
                );
                return;
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
        if let Ty::Named { name, .. } = ty {
            let written = self.global(&crate::def::impl_name(Some(trait_name), name));
            let listed = (self.env.impls(name).into_iter())
                .filter(|(t, _)| t.as_deref().and_then(|t| t.split('<').next()) == Some(trait_name))
                .map(|(_, d)| d);
            for def in written.into_iter().chain(listed) {
                if let Def::Impl {
                    trait_args,
                    target,
                    bound_args,
                    ..
                } = &*def
                {
                    if trait_args.len() == n {
                        let mut on = crate::ty::Bindings::default();
                        target.unify(ty, &mut on);
                        // `impl<S: Indexed<K, T>, K, T> Seq<T> for Rebased<S>`: `T` is what `S` gives.
                        for (param, bound, args) in bound_args {
                            let Some(actual) = on.get(param).cloned() else {
                                continue;
                            };
                            let Some(given) = self.trait_args_of(&actual, bound, args.len()) else {
                                continue;
                            };
                            let mut solved = crate::ty::Bindings::default();
                            for (want, got) in args.iter().zip(&given) {
                                want.unify(got, &mut solved);
                            }
                            for (g, t) in solved {
                                on.entry(g).or_insert(t);
                            }
                        }
                        return Some(trait_args.iter().map(|t| t.subst(&on)).collect());
                    }
                }
            }
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
            // A range is indexed by position, `(lo..hi)[i]` (R07.3.5).
            Ty::Named { name, .. } if name == "List" || name == "Range" => Ty::list_index(),
            _ => Ty::Unknown,
        };
        let mut out = vec![Ty::Unknown; n - 1];
        if let Some(last) = out.last_mut() {
            *last = key;
        }
        out.push(elem);
        Some(out)
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
