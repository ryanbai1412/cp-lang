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
        let mut bound_args = self.quietly(|c| c.bound_args(bounds));
        // A plain bound (`impl<T: Mul + One>`) holds too, with no arguments to bind.
        for b in bounds {
            for t in &b.traits {
                if let ast::TypeKind::Named { name, args } = &t.kind {
                    if args.is_empty() {
                        bound_args.push((b.name.text.clone(), name.text.clone(), Vec::new()));
                    }
                }
            }
        }
        (trait_args, bound_args)
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

    /// The generics a bare bound may bind by name: not one only a `-> yields`
    /// result names, which takes what the body yields (R07.2.10).
    pub(crate) fn bound_bindable(
        generics: &[String],
        params: &[crate::ty::Param],
        ret: &Ty,
    ) -> Vec<String> {
        let generator = matches!(ret, Ty::Named { name, .. } if name == "Gen");
        let named = |g: &String| {
            let mut hit = false;
            for p in params {
                p.ty.map(&mut |t| {
                    hit |= matches!(t, Ty::Param(n) if n == g);
                    None
                });
            }
            hit
        };
        generics
            .iter()
            .filter(|g| !generator || named(g))
            .cloned()
            .collect()
    }

    /// `last<S: Bidir, T>(w)`: a trait parameter the fn leaves free is the
    /// one the argument's impl of the bound gives it, by name; a bound written
    /// with its arguments (`from: Seq<V>`) binds those instead.
    pub(crate) fn bind_bound_params(
        &mut self,
        generics: &[String],
        (bounds, written): (&[(String, Vec<String>)], &BoundArgs),
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
                let found = self.type_def(trait_name);
                let Some(Def::Trait {
                    generics: of_trait, ..
                }) = found.as_deref()
                else {
                    continue;
                };
                let pattern = written
                    .iter()
                    .find(|(p, t, args)| p == param && t == trait_name && !args.is_empty())
                    .map(|(_, _, args)| args.clone())
                    .unwrap_or_else(|| of_trait.iter().cloned().map(Ty::Param).collect());
                let free = |g: &String, b: &crate::ty::Bindings| {
                    generics.contains(g) && !b.contains_key(g)
                };
                let named = pattern.iter().flat_map(params_named).collect::<Vec<_>>();
                if !named.iter().any(|g| free(g, bindings)) {
                    continue;
                }
                let Some(args) = self.trait_args_of(&on_ty, trait_name, pattern.len()) else {
                    continue;
                };
                let mut got = bindings.clone();
                for (want, arg) in pattern.iter().zip(&args) {
                    want.unify(arg, &mut got);
                }
                for g in named {
                    if let Some(t) = got.get(&g).filter(|_| free(&g, bindings)).cloned() {
                        bindings.insert(g, t);
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
        let found = self.type_def(trait_name);
        let Some(Def::Trait {
            methods,
            supers,
            super_args,
            ..
        }) = found.as_deref()
        else {
            return None;
        };
        if let Some(m) = methods.iter().find(|m| m.name == name) {
            return Some(m.ty.subst(&on));
        }
        for (sup, args) in supers.iter().zip(super_args) {
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

    /// The type of member `name` of `param.assoc`, from the traits its declaration bounds it
    /// by: inside `fn f<G: Next<V>>`, an arc `a: G.Arc` has `Target<V>`'s `to`.
    pub(crate) fn assoc_bound_member(
        &mut self,
        param: &str,
        assoc: &str,
        name: &str,
    ) -> Option<Ty> {
        let of = Ty::Assoc {
            of: Box::new(Ty::Param(param.into())),
            name: assoc.into(),
        };
        let bounds: Vec<String> = (self.param_bounds.iter())
            .filter(|(p, _)| p == param)
            .flat_map(|(_, ts)| ts.iter().cloned())
            .collect();
        for trait_name in bounds {
            let Some(def) = self.type_def(&trait_name) else {
                continue;
            };
            let Def::Trait {
                assoc: declared, ..
            } = &*def
            else {
                continue;
            };
            let Some((_, assoc_bounds, _)) = declared.iter().find(|(n, ..)| n == assoc) else {
                continue;
            };
            let outer = self.bound_bindings(param, &trait_name);
            for (bound, args) in assoc_bounds {
                let Some(def) = self.type_def(bound) else {
                    continue;
                };
                let Def::Trait {
                    generics, methods, ..
                } = &*def
                else {
                    continue;
                };
                let Some(m) = methods.iter().find(|m| m.name == name) else {
                    continue;
                };
                let mut on = crate::ty::Bindings::default();
                on.insert("Self".into(), of.clone());
                for (g, a) in generics.iter().zip(args) {
                    on.insert(g.clone(), a.subst(&outer));
                }
                return Some(super::assoc::read_as(m, m.ty.subst(&on)));
            }
        }
        None
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
        if let Def::Fn {
            bounds, bound_args, ..
        } = def
        {
            let mut all = bound_args.clone();
            for (p, traits) in bounds {
                for t in traits {
                    if !all.iter().any(|(q, w, _)| q == p && w == t) {
                        all.push((p.clone(), t.clone(), Vec::new()));
                    }
                }
            }
            self.fn_items_take_bound_types(&all, params, args, bindings);
        }
    }

    /// Each bound a fn item's own type meets, with the trait arguments the call gave it.
    fn note_item_args(&mut self, bound_args: &BoundArgs, bindings: &crate::ty::Bindings) {
        for (param, trait_name, args) in bound_args {
            if let Some(Ty::Named { name, .. }) = bindings.get(param) {
                let known: Vec<Ty> = args.iter().map(|a| a.subst(bindings)).collect();
                if name.starts_with("type_of(")
                    && !known.iter().any(|a| a.mentions_param() || a.has_vague())
                {
                    self.item_args
                        .push((name.clone(), trait_name.clone(), known));
                }
            }
        }
    }

    /// [`Self::fn_items_take_their_type`] under `bound_args`.
    pub(crate) fn fn_items_take_bound_types(
        &mut self,
        bound_args: &BoundArgs,
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
            let Some(item) = arg.value.fn_item_type() else {
                continue;
            };
            if matches!(&arg.value.kind, ast::ExprKind::Name(n) if !self.names_fn_item(n)) {
                continue;
            }
            let has_impl = self.env.impls(&item).iter().any(|(t, _)| {
                let t = t.as_deref().and_then(|t| t.split('<').next());
                bound_args.iter().any(|(p, w, _)| p == m && Some(&**w) == t)
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
        self.note_item_args(bound_args, bindings);
    }

    /// At a call: each generic a bound's trait arguments name, from the
    /// type the bound's parameter is bound to, so `sum(xs)` over
    /// `S: Seq<T>` given a `List<Int>` has `T = Int`.
    pub(crate) fn bind_bound_args(&mut self, bindings: &mut crate::ty::Bindings) {
        let Some(bound_args) = self.call_bound_args.take() else {
            return;
        };
        self.note_item_args(&bound_args, bindings);
        for (param, trait_name, args) in &bound_args {
            let Some(actual) = bindings.get(param).cloned() else {
                continue;
            };
            let actual = self.settle(&actual);
            let item = matches!(&actual, Ty::Named { name, .. } if name.starts_with("type_of("));
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
                    // An impl generic nothing fixed yet (`impl<T> Op<T> for type_of(max)`) waits.
                    let got = self.settle(&got.subst(&open));
                    if !(item && got.has_vague()) {
                        want.unify(&got, bindings);
                    }
                }
            }
        }
        self.call_bound_args = Some(bound_args);
    }

    /// An item bound to `M: Op<T>` has its impl's own bounds met at the `T` the call gives:
    /// `impl<T: Mul + One> Op<T> for type_of(*)` takes `q(*, 3)`, not `q(*, (1, 2))`.
    pub(crate) fn item_impls_hold(
        &mut self,
        fn_name: &str,
        bound_args: &BoundArgs,
        bindings: &crate::ty::Bindings,
        span: Span,
    ) {
        for (param, trait_name, args) in bound_args {
            let Some(Ty::Named { name: item, .. }) = bindings.get(param).map(|t| self.settle(t))
            else {
                continue;
            };
            if !item.starts_with("type_of(") {
                continue;
            }
            let want: Vec<Ty> = (args.iter())
                .map(|a| self.settle(&a.subst(bindings)))
                .collect();
            if want.iter().any(|t| t.has_vague() || t.mentions_param())
                || self.item_impl_holds(&item, trait_name, &want)
            {
                continue;
            }
            let want = want
                .iter()
                .map(Ty::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            self.report(
                span,
                msgs::check::MISSING_IMPL_DOES_NOT_IMPLEMENT_REQUIRED,
                &[&item, &format!("{trait_name}<{want}>"), &fn_name],
            );
        }
    }

    /// Whether one of `item`'s impls of `trait_name` meets its head's bounds with its trait
    /// arguments bound to `want`.
    fn item_impl_holds(&mut self, item: &str, trait_name: &str, want: &[Ty]) -> bool {
        let impls: Vec<_> = (self.env.impls(item).into_iter())
            .filter(|(t, _)| t.as_deref().and_then(|t| t.split('<').next()) == Some(trait_name))
            .map(|(_, d)| d)
            .collect();
        impls.is_empty()
            || impls.iter().any(|def| {
                let Def::Impl {
                    trait_args,
                    bound_args,
                    ..
                } = &**def
                else {
                    return true;
                };
                let mut on = crate::ty::Bindings::default();
                for (a, w) in trait_args.iter().zip(want) {
                    a.unify(w, &mut on);
                }
                let mut seen = Vec::new();
                bound_args.iter().all(|(p, b, _)| {
                    on.get(p).cloned().is_none_or(|t| {
                        t.has_vague() || self.implements_trait_inner(&t, b, &mut seen)
                    })
                })
            })
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
                if self.fits(got, first) || self.fits(first, got) {
                    continue;
                }
                let shown = |(p, t, a): &(String, String, Vec<Ty>)| {
                    let (name, args) = (t.clone(), a.clone());
                    format!("{p}: {}", Ty::Named { name, args })
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
                    let mut on = crate::ty::Bindings::default();
                    target.unify(ty, &mut on);
                    if trait_args.len() == n && ty.fits(&target.subst(&on)) {
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

/// The type parameters `ty` names.
pub(crate) fn params_named(ty: &Ty) -> Vec<String> {
    let mut out = Vec::new();
    ty.map(&mut |t| {
        if let Ty::Param(p) = t {
            out.push(p.clone());
        }
        None
    });
    out
}
