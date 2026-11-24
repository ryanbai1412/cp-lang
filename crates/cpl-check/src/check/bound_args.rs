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

    /// Each method's [`MethodRows`], its trait bounds joined by `outer`, its impl's.
    pub(super) fn method_rows(
        &mut self,
        methods: &[ast::Stmt],
        outer: &[ast::Bound],
    ) -> Vec<crate::def::MethodRows> {
        let mut impl_traits = Vec::new();
        for b in outer {
            for t in &b.traits {
                if let ast::TypeKind::Named { name, .. } = &t.kind {
                    impl_traits.push((b.name.text.clone(), name.text.clone()));
                }
            }
        }
        (methods.iter())
            .filter_map(|m| match &m.kind {
                ast::StmtKind::Fn(f) => {
                    let decl = self.trait_params_generic(f);
                    let (rows, mut traits, through) = self.quietly(|c| c.fn_bounds(&decl));
                    traits.extend(impl_traits.iter().cloned());
                    Some((f.name.text.clone(), rows, through, traits))
                }
                _ => None,
            })
            .collect()
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
        let args = self.bound_trait_args(param, trait_name);
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
            let on = self.bound_bindings(param, &trait_name);
            if let Some(ty) = self.trait_method(&trait_name, name, on, &mut Vec::new()) {
                return Some(ty);
            }
        }
        None
    }

    /// A trait's method, its own or a super's, the super's args read as
    /// written on the trait (`Table<T>: Positional<T>` reaches `len`).
    fn trait_method(
        &mut self,
        trait_name: &str,
        name: &str,
        on: crate::ty::Bindings,
        seen: &mut Vec<String>,
    ) -> Option<Ty> {
        if seen.iter().any(|s| s == trait_name) {
            return None;
        }
        seen.push(trait_name.to_string());
        let Some(Def::Trait {
            methods,
            supers,
            super_args,
            ..
        }) = self.type_def(trait_name).as_deref().cloned()
        else {
            return None;
        };
        if let Some(m) = methods.iter().find(|m| m.name == name) {
            return Some(m.ty.subst(&on));
        }
        for (sup, args) in supers.iter().zip(&super_args) {
            let mut theirs = crate::ty::Bindings::default();
            if let Some(this) = on.get("Self") {
                theirs.insert("Self".into(), this.clone());
            }
            if let Some(Def::Trait { generics, .. }) = self.type_def(sup).as_deref() {
                for (g, a) in generics.iter().zip(args) {
                    theirs.insert(g.clone(), a.subst(&on));
                }
            }
            if let Some(ty) = self.trait_method(sup, name, theirs, seen) {
                return Some(ty);
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
            if matches!(&arg.value.kind, ast::ExprKind::Name(n) if !self.names_fn_item(n)) {
                continue;
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
        self.bind_written_bound_args(&bound_args, bindings);
        self.call_bound_args = Some(bound_args);
    }

    /// [`Self::bind_bound_args`] for the bounds `bound_args` writes:
    /// `S: Indexed<K, T>` given a `Slice<List<Int>>` has `K = Int`.
    pub(crate) fn bind_written_bound_args(
        &mut self,
        bound_args: &crate::def::BoundArgs,
        bindings: &mut crate::ty::Bindings,
    ) {
        for (param, trait_name, args) in bound_args {
            let Some(actual) = bindings.get(param).cloned() else {
                continue;
            };
            let actual = self.settle(&actual);
            if let Some(given) = self.trait_args_of(&actual, trait_name, args.len()) {
                // `impl<T> Acts<T, T> for Plus`: the impl's own `T` is the
                // bound's, where the call already fixed it.
                let mut open = crate::ty::Bindings::default();
                for (want, got) in args.iter().zip(&given) {
                    let known = self.bound_assoc(want, &bound_args, bindings);
                    let known = self.resolve_assoc(&known);
                    if !known.mentions_param() && !known.has_assoc() {
                        got.unify(&known, &mut open);
                    }
                }
                // `A: Acts<M.Fold, F>` before `M` is known leaves the impl's `T` open.
                for (want, got) in args.iter().zip(&given) {
                    let got = got.subst(&open);
                    if !got.mentions_param() {
                        want.unify(&got, bindings);
                    }
                }
            }
        }
    }

    /// `M.Fold` for `M: Op<T>` bound to `NoOp`: the impl's choice, with the
    /// impl's own `T` taken from the bound's.
    fn bound_assoc(
        &mut self,
        want: &Ty,
        bound_args: &BoundArgs,
        bindings: &crate::ty::Bindings,
    ) -> Ty {
        if let Ty::Assoc { of, name } = want {
            if let Some((p, actual)) = match of.as_ref() {
                Ty::Param(p) => bindings.get(p).map(|a| (p, a)),
                _ => None,
            } {
                let actual = self.settle(actual);
                if let Some(chosen) = self.project(&actual, name) {
                    let mut open = crate::ty::Bindings::default();
                    for (_, tr, args) in bound_args.iter().filter(|(q, ..)| q == p) {
                        let given = self
                            .trait_args_of(&actual, tr, args.len())
                            .unwrap_or_default();
                        for (w, g) in args.iter().zip(given) {
                            g.unify(&w.subst(bindings), &mut open);
                        }
                    }
                    return chosen.subst(&open).subst(bindings);
                }
            }
        }
        want.subst(bindings)
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
        let mut seen: Vec<(&String, usize, Ty)> = Vec::new();
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
                let Some(&(_, j, ref first)) = seen.iter().find(|(h, ..)| *h == g) else {
                    seen.push((g, i, got.clone()));
                    continue;
                };
                if got.fits(first) || first.fits(got) {
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

    /// What an impl for `target` binds its generics to on `ty`, those its
    /// bounds fix included: `impl<S: Indexed<K, T>, K, T> Seq<T> for
    /// Rebased<S>` on a `Rebased<List<Int>>` has `T = Int`.
    fn impl_on(
        &mut self,
        target: &Ty,
        bound_args: &crate::def::BoundArgs,
        ty: &Ty,
    ) -> crate::ty::Bindings {
        let mut on = crate::ty::Bindings::default();
        target.unify(ty, &mut on);
        self.bind_written_bound_args(bound_args, &mut on);
        on
    }

    /// The generics `ty`'s impls fix for the members they give it: on a
    /// `Slice<[V]Int>`, `at(self, i: K)` takes a `V`.
    pub(crate) fn impl_bindings(&mut self, ty: &Ty, bindings: &mut crate::ty::Bindings) {
        let Ty::Named { name, .. } = ty else {
            return;
        };
        for (_, def) in self.env.impls(name) {
            if let Def::Impl {
                target, bound_args, ..
            } = &*def
            {
                let on = self.impl_on(target, bound_args, ty);
                for (g, t) in on {
                    bindings.entry(g).or_insert(t);
                }
            }
        }
    }

    /// The arguments `param`'s bounds give trait `want`, directly or through a
    /// supertrait: `S: Indexed<K, E>` gives `Seq` the argument `E`.
    pub(crate) fn bound_trait_args(&mut self, param: &str, want: &str) -> Option<Vec<Ty>> {
        let mut stack: Vec<(String, Vec<Ty>)> = (self.param_bound_args.iter())
            .filter(|(p, _, _)| p == param)
            .map(|(_, t, args)| (t.clone(), args.clone()))
            .collect();
        let mut seen = Vec::new();
        while let Some((trait_name, args)) = stack.pop() {
            if trait_name == want {
                return Some(args);
            }
            if seen.contains(&trait_name) {
                continue;
            }
            if let Some(Def::Trait {
                generics,
                supers,
                super_args,
                ..
            }) = self.type_def(&trait_name).as_deref()
            {
                let on: crate::ty::Bindings =
                    generics.iter().cloned().zip(args.iter().cloned()).collect();
                let supers = supers.iter().zip(super_args);
                stack.extend(
                    supers.map(|(s, a)| (s.clone(), a.iter().map(|t| t.subst(&on)).collect())),
                );
            }
            seen.push(trait_name);
        }
        None
    }

    /// The arguments a written impl, or a parameter's bound, gives
    /// `trait_name` for `ty`; never asks whether `ty` implements it, so the
    /// `Seq` walk check can use it without recursing.
    pub(crate) fn impl_trait_args(
        &mut self,
        ty: &Ty,
        trait_name: &str,
        n: usize,
    ) -> Option<Vec<Ty>> {
        if let Ty::Param(p) = ty {
            return self
                .bound_trait_args(p, trait_name)
                .filter(|args| args.len() == n);
        }
        use super::impl_head::ty_impl_name;
        if let Some(name) = ty_impl_name(ty) {
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
                    let on = self.impl_on(target, bound_args, ty);
                    if trait_args.len() == n && ty.fits(&target.subst(&on)) {
                        return Some(trait_args.iter().map(|t| t.subst(&on)).collect());
                    }
                }
            }
        }
        None
    }

    /// The arguments `ty` gives trait `trait_name`: a bound's in a generic
    /// body, an `impl`'s as written, or a built-in sequence's key and
    /// element.
    pub(crate) fn trait_args_of(&mut self, ty: &Ty, trait_name: &str, n: usize) -> Option<Vec<Ty>> {
        if let Some(args) = self.impl_trait_args(ty, trait_name, n) {
            return Some(args);
        }
        if let Ty::Param(_) = ty {
            return None;
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
