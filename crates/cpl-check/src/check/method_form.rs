//! `xs.f(args)` on a library `fn f(self: S, ...)` checks as `f(xs, args)`.
//! R11.3.4: every top-level `fn f(self: ..)` in reach, the file's own and
//! each module's, is a candidate, and the call takes the one whose `self`
//! its receiver fits. The choice is the `Resolved` at the method's name,
//! which lowering reads.

use super::*;
use cpl_diag::msgs;
use cpl_syntax::ast::{self, Arg, Expr, ExprKind};
use std::sync::Arc;

/// Which candidate a receiver picks.
enum Pick {
    One(String, Arc<Def>),
    Ambiguous(Vec<String>),
    /// The receiver's type is not known well enough to say.
    Unsettled,
    /// The receiver's type is known and no candidate's `self` takes it.
    NoFit,
}

fn takes_self(def: &Def) -> bool {
    matches!(def, Def::Fn { ty: Ty::Fn { params, .. }, .. }
        if params.first().is_some_and(|p| p.name == "self" && !p.named_only))
}

/// Whether `def` takes `n` positional arguments after `self`: one never
/// binds a named-only parameter, so `xs.sorted(x => -x)` isn't `sorted(by:)`.
fn takes_positionals(def: &Def, n: usize) -> bool {
    matches!(def, Def::Fn { ty: Ty::Fn { params, .. }, .. }
        if params.iter().skip(1).filter(|p| !p.named_only).count() >= n)
}

fn self_ty(def: &Def) -> Option<Ty> {
    match def {
        Def::Fn {
            ty: Ty::Fn { params, .. },
            ..
        } => params.first().map(|p| p.ty.clone()),
        _ => None,
    }
}

/// Whether `to` is `from` with its type parameters bound: `Range<D>` to
/// `Range<F64>`.
fn instance_of(to: &Ty, from: &Ty) -> bool {
    let mut bindings = crate::ty::Bindings::default();
    from.unify(to, &mut bindings);
    from.subst(&bindings) == *to
}

/// A candidate's module, and its `#k` among one module's overloads.
type Origin = (Option<String>, Option<String>);

impl Checker<'_> {
    /// Every top-level `fn name(self: ..)` in reach, by the name it is
    /// imported under, one per module.
    pub(super) fn method_candidates(&mut self, name: &str) -> Vec<(String, Arc<Def>)> {
        let mut out: Vec<(String, Arc<Def>, Origin)> = Vec::new();
        let keys = std::iter::once(name.to_string()).chain(self.env.method_keys(name));
        for key in keys {
            let Some(def) = self.global(&key).filter(|d| takes_self(d)) else {
                continue;
            };
            let module = if self.local_defs.contains_key(&key) {
                None
            } else {
                self.env.origin(&key)
            };
            let overload = prelude::overload_part(&key).map(|(_, k)| k.to_string());
            let origin = (module, overload);
            if !out.iter().any(|(_, _, o)| *o == origin) {
                out.push((key, def, origin));
            }
        }
        out.into_iter().map(|(k, d, _)| (k, d)).collect()
    }

    /// R08.1.4: a later `fn name(self: ..)` of this module whose `self` is
    /// neither more nor less specific than an earlier one's is E0702.
    pub(super) fn overload_clash(&mut self, f: &ast::FnDecl) {
        let Some(here) = self.env.item_index() else {
            return;
        };
        let name = &f.name.text;
        let own: Vec<String> = std::iter::once(name.clone())
            .chain(self.env.method_keys(name))
            .filter(|k| {
                k == name
                    || prelude::overload_part(k)
                        .is_some_and(|(_, m)| k.ends_with(&format!(" of {m}")))
            })
            .collect();
        let at = |c: &mut Self, k: &str| c.env.decl_order(k);
        let Some(mine) = own.iter().find(|k| at(self, k) == Some(here)).cloned() else {
            return;
        };
        let Some(mine_def) = self.global(&mine) else {
            return;
        };
        for key in own.iter().filter(|k| **k != mine) {
            if !at(self, key).is_some_and(|i| i < here) {
                continue;
            }
            let Some(that_def) = self.global(key) else {
                continue;
            };
            // R06.1.6: other required labels make it an overload of its own.
            if def_labels(&mine_def) != def_labels(&that_def) {
                continue;
            }
            if self.overlaps(&mine_def, &that_def)
                && !self.beats(&mine_def, &that_def)
                && !self.beats(&that_def, &mine_def)
            {
                self.report(
                    f.name.span,
                    msgs::check::OVERLOAD_BY_TYPE_A_SECOND_FN_WHOSE,
                    &[&name],
                );
                return;
            }
        }
    }

    /// R08.1.4: `a` is more specific than `b` when `b`'s `self` binds to
    /// `a`'s and not the other way, or `b`'s is a bare `self: S` whose
    /// bounds `a`'s concrete `self` meets: `self: Str` beats `self: S` with
    /// `S: Indexed<D, T>`.
    fn beats(&mut self, a: &Def, b: &Def) -> bool {
        let (Some(sa), Some(sb)) = (self_ty(a), self_ty(b)) else {
            return false;
        };
        match (&sa, &sb) {
            (Ty::Param(_), _) => false,
            (_, Ty::Param(_)) => self.self_fits(&sa, b),
            _ => instance_of(&sa, &sb) && !instance_of(&sb, &sa),
        }
    }

    /// `def`'s positional parameter types once its `self` is `on`, with each
    /// generic its bounds' trait arguments name bound from `on`: `x: T` of
    /// `S: Indexed<D, T>` is `x: Char` on a `Str`.
    fn params_on(&mut self, def: &Def, on: &Ty) -> Vec<Ty> {
        let Def::Fn {
            ty: Ty::Fn { params, .. },
            bound_args,
            ..
        } = def
        else {
            return Vec::new();
        };
        let mut bindings = crate::ty::Bindings::default();
        if let Some(first) = params.first() {
            first.ty.unify(on, &mut bindings);
        }
        for (param, trait_name, want) in bound_args {
            let Some(actual) = bindings.get(param).cloned() else {
                continue;
            };
            if let Some(given) = self.trait_args_of(&actual, trait_name, want.len()) {
                for (w, g) in want.iter().zip(&given) {
                    w.unify(g, &mut bindings);
                }
            }
        }
        (params.iter().skip(1))
            .filter(|p| !p.named_only)
            .map(|p| p.ty.subst(&bindings))
            .collect()
    }

    /// R08.1.4: whether one call can take both `a` and `b`: an argument count
    /// both take, and each parameter's type the other's could be, once `b`'s
    /// `self` is `a`'s and the other way. `find(self: Str, pat: Str)` and
    /// `find<S: Indexed<D, T>, T>(self: S, x: T)` don't: on a `Str`, `x` is a `Char`.
    fn overlaps(&mut self, a: &Def, b: &Def) -> bool {
        let max = |d: &Def| match d {
            Def::Fn {
                ty: Ty::Fn { params, .. },
                ..
            } => params.len(),
            _ => 0,
        };
        if !(0..=max(a).max(max(b))).any(|n| takes_count(a, n) && takes_count(b, n)) {
            return false;
        }
        [(a, b), (b, a)].into_iter().all(|(a, b)| {
            let Some(on) = self_ty(a) else {
                return true;
            };
            if !on.mentions_param() && !self.self_fits(&on, b) {
                return false;
            }
            let (pa, pb) = (self.params_on(a, &on), self.params_on(b, &on));
            pa.iter().zip(&pb).all(|(x, y)| {
                if self.bound_refuses(a, x, y) || self.bound_refuses(b, y, x) {
                    return false;
                }
                x.mentions_param() || y.mentions_param() || self.fits(x, y) || self.fits(y, x)
            })
        })
    }

    /// Whether `def`'s parameter of generic type `x` can't be the other's
    /// `y`, for a bound `y` lacks: `x: T` with `T: Eq` takes no `T -> Bool`.
    fn bound_refuses(&mut self, def: &Def, x: &Ty, y: &Ty) -> bool {
        let (Ty::Param(g), Def::Fn { bounds, .. }) = (x, def) else {
            return false;
        };
        if matches!(y, Ty::Param(_)) {
            return false;
        }
        let traits: Vec<String> = (bounds.iter())
            .filter(|(p, _)| p == g)
            .flat_map(|(_, t)| t.iter().cloned())
            .collect();
        traits.iter().any(|t| !self.implements_trait(y, t))
    }

    /// Whether `def`'s parameters, on `base`, take the types `args` have,
    /// where both are known: `"ab".find('a')` is no call of `find(self: Str, pat: Str)`.
    fn takes_arg_types(&mut self, def: &Def, base: &Ty, args: &[Arg]) -> bool {
        let params = self.params_on(def, base);
        let positional = args.iter().filter(|a| a.name.is_none());
        positional.zip(params).all(|(arg, want)| {
            if matches!(arg.value.kind, ExprKind::Lambda { .. }) {
                let lambda = Ty::Fn {
                    params: Vec::new(),
                    ret: Box::new(Ty::Unknown),
                };
                return match want {
                    Ty::Fn { .. } => true,
                    Ty::Param(_) => !self.bound_refuses(def, &want, &lambda),
                    _ => want.has_vague(),
                };
            }
            if want.mentions_param() || want.has_vague() {
                return true;
            }
            let got = self.quietly(|c| c.infer(&arg.value));
            got.has_vague() || got.mentions_param() || self.fits(&got, &want)
        })
    }

    /// The base of a field read is a value, not the callee, even of
    /// `xs.ring.windows(2)`.
    pub(super) fn infer_base(&mut self, base: &Expr) -> Ty {
        let callee = std::mem::replace(&mut self.in_call_callee, false);
        let ty = self.infer(base);
        self.in_call_callee = callee;
        ty
    }

    /// `base.name`'s type: a member, else a `@get fn` of that name.
    pub(super) fn member_or_getter(&mut self, expr: &Expr, name: &ast::Name, base: Ty) -> Ty {
        // A getter read is a call: its facts say which fn it calls.
        if !self.in_call_callee && !self.exprs.calls.iter().any(|c| c.call == expr.span) {
            if self.builtin_get(&base, &name.text) {
                self.record_getter(expr, expr, true);
            } else if self.names_getter_fn(&base, &name.text) {
                self.record_getter(expr, expr, false);
            }
        }
        match self.member_ty(expr, name, base) {
            Ty::Unknown if !self.in_call_callee => self.getter_fn_read(expr).unwrap_or(Ty::Unknown),
            ty => ty,
        }
    }

    /// Whether `base.name` names a `@get fn`, std's `s.first` on a `SortedBag`.
    pub(super) fn names_getter_fn(&mut self, base: &Ty, name: &str) -> bool {
        if let Ty::Named { name: owner, .. } = base {
            if let Some(members) = self.members_of(owner) {
                if members.fields.iter().any(|f| f.name == name) {
                    return false;
                }
                if members.members.iter().any(|(m, _)| m == name) {
                    return true;
                }
            }
        }
        matches!(self.pick_method(base, name), Pick::One(_, def) if matches!(&*def, Def::Fn { getter: true, .. }))
    }

    /// `xs.ring`: a field read naming a `@get fn` is the call `ring(xs)`.
    pub(super) fn getter_fn_read(&mut self, expr: &Expr) -> Option<Ty> {
        let ExprKind::Field { base, name } = &expr.kind else {
            return None;
        };
        let base_ty = self.infer_quietly(base);
        let Pick::One(_, def) = self.pick_method(&base_ty, &name.text) else {
            return None;
        };
        if !matches!(&*def, Def::Fn { getter: true, .. }) {
            return None;
        }
        // A call giving a value of a struct leaves its type in `exprs.types`.
        let read = |c: &mut Self| {
            let ty = c.method_form_call(expr, expr, &[]);
            ty.or_else(|| c.exprs.last_type(expr.span).cloned())
        };
        let settled = self.quietly(|c| {
            read(c).filter(|t| c.sink.is_empty() && !t.has_vague() && !names_param(t))
        });
        settled.as_ref()?;
        // The read's hover is the value it reads, not the fn it calls.
        let hovers = self.hovers.len();
        let ty = read(self);
        let mut at = hovers;
        while at < self.hovers.len() {
            if self.hovers[at].span == name.span {
                self.hovers.remove(at);
            } else {
                at += 1;
            }
        }
        ty
    }

    /// Whether `base` can be the `self` of the fn `def`: it fits the
    /// parameter's type, and a bare type parameter's bounds.
    fn self_fits(&mut self, base: &Ty, def: &Def) -> bool {
        let Def::Fn {
            ty: Ty::Fn { params, .. },
            bounds,
            ..
        } = def
        else {
            return false;
        };
        let Some(first) = params.first() else {
            return false;
        };
        if let Ty::Param(g) = &first.ty {
            let traits: Vec<String> = (bounds.iter())
                .filter(|(p, _)| p == g)
                .flat_map(|(_, t)| t.iter().cloned())
                .collect();
            return traits.iter().all(|t| self.implements_trait(base, t));
        }
        let same_shape = match (&first.ty, base) {
            (Ty::Record(takes), Ty::Record(on)) => {
                takes.len() == on.len() && takes.iter().zip(on).all(|(a, b)| a.0 == b.0)
            }
            (Ty::Tuple(takes), Ty::Tuple(on)) => takes.len() == on.len(),
            (Ty::Tuple(takes), Ty::Record(on)) => takes.len() == on.len(),
            (Ty::Tuple(takes), Ty::Pos(d)) => {
                (self.product_factors(d)).is_some_and(|parts| parts.len() == takes.len())
            }
            (Ty::Named { name: takes, .. }, Ty::Named { name: on, .. }) => takes == on,
            _ => false,
        };
        // `self: [D]T` takes `List<Int>` with `D` and `T` bound to its parts.
        let mut bindings = crate::ty::Bindings::default();
        let base = &self.narrowed(base, &first.ty);
        first.ty.unify(base, &mut bindings);
        let takes = first.ty.subst(&bindings);
        // `self: Range<F64>` has `Range<Int>`'s shape but doesn't take it.
        let named = matches!(&first.ty, Ty::Named { name, .. } if name != "Gen");
        (same_shape && (!named || self.fits(base, &takes))) || (!named && self.fits(base, &takes))
    }

    /// The candidate `base` fits; of several, one whose `self` is a type of
    /// its own wins over one whose `self` is any type a bound allows.
    fn pick_method(&mut self, base: &Ty, name: &str) -> Pick {
        self.pick_method_for(base, name, None)
    }

    /// As `pick_method`, also dropping a candidate whose parameters can't
    /// take `args`' shapes: `s.split(sep: c => c == ' ')` on a `Str` is the
    /// predicate split, not `split(self: Str, sep: Char)`.
    fn pick_method_for(&mut self, base: &Ty, name: &str, args: Option<&[Arg]>) -> Pick {
        if base.is_vague() || matches!(base, Ty::Param(_) | Ty::Error) {
            return Pick::Unsettled;
        }
        let mut fit: Vec<(String, Arc<Def>)> = Vec::new();
        for (key, def) in self.method_candidates(name) {
            if self.self_fits(base, &def)
                && args.is_none_or(|args| takes_shapes(&def, args) && def_labels_fit(&def, args))
            {
                fit.push((key, def));
            }
        }
        // R11.2.3: a name a `use` imports shadows the prelude's, so the
        // prelude's candidates drop out beside one. A fn the file declares
        // stands beside them, and specificity picks (decision 486).
        let shadows: Vec<bool> = fit.iter().map(|f| self.env.chosen(&f.0)).collect();
        if shadows.contains(&true) {
            let mut shadows = shadows.into_iter();
            fit.retain(|_| shadows.next().unwrap_or(false));
        }
        // Of several, those whose parameters take the arguments' types.
        if let Some(args) = args.filter(|_| fit.len() > 1) {
            let defs: Vec<Arc<Def>> = fit.iter().map(|f| f.1.clone()).collect();
            let takes: Vec<bool> = (defs.iter())
                .map(|d| self.takes_arg_types(d, base, args))
                .collect();
            if takes.contains(&true) {
                let mut takes = takes.into_iter();
                fit.retain(|_| takes.next().unwrap_or(false));
            }
        }
        // R08.1.4: of the candidates, those no other is more specific than.
        let defs: Vec<Arc<Def>> = fit.iter().map(|f| f.1.clone()).collect();
        let beaten: Vec<bool> = (0..defs.len())
            .map(|b| (0..defs.len()).any(|a| a != b && self.beats(&defs[a], &defs[b])))
            .collect();
        let mut beaten = beaten.into_iter();
        fit.retain(|_| !beaten.next().unwrap_or(false));
        // Of candidates the receiver fits alike, those the argument count fits.
        if let Some(args) = args.filter(|_| fit.len() > 1) {
            let given = args.iter().filter(|a| a.name.is_none()).count();
            if fit.iter().any(|(_, d)| takes_count(d, given)) {
                fit.retain(|(_, d)| takes_count(d, given));
            }
        }
        // With no call to read labels from, the overload that needs none.
        if args.is_none() && fit.len() > 1 && fit.iter().any(|(_, d)| def_labels(d).is_empty()) {
            fit.retain(|(_, d)| def_labels(d).is_empty());
        }
        if fit.len() > 1 {
            return Pick::Ambiguous(fit.into_iter().map(|f| f.0).collect());
        }
        match fit.pop() {
            None => Pick::NoFit,
            Some((key, def)) => Pick::One(key, def),
        }
    }

    /// Whether `base.name` picks a fn: `None` when no candidate's `self`
    /// takes `base` as it is (a `self: ?T` takes a `T` only by lifting it,
    /// a `self: (R, C)` a record only by reading it as a tuple), else
    /// whether the one it picks takes any type (`fn f<T>(self: T)`, `T`
    /// unbounded).
    pub(crate) fn method_fit(&mut self, base: &Ty, name: &str) -> Option<bool> {
        let coerced = |def: &Def| match (self_ty(def), base) {
            (Some(Ty::Optional(_)), b) => !matches!(b, Ty::Optional(_)),
            (Some(Ty::Tuple(_)), Ty::Record(_)) => true,
            _ => false,
        };
        match self.pick_method(base, name) {
            Pick::One(_, def) if coerced(&def) => None,
            Pick::One(_, def) => Some(match (&*def, self_ty(&def)) {
                (Def::Fn { bounds, .. }, Some(Ty::Param(g))) => {
                    (bounds.iter()).all(|(p, t)| *p != g || t.is_empty())
                }
                _ => false,
            }),
            Pick::Ambiguous(_) => Some(false),
            Pick::Unsettled | Pick::NoFit => None,
        }
    }

    /// The signature struct `ty_name` declares for its method `method`,
    /// `self` left out, with the struct's type parameters.
    pub(crate) fn struct_method(
        &mut self,
        ty_name: &str,
        method: &str,
    ) -> Option<(Vec<String>, Ty)> {
        match self.type_def(ty_name).as_deref() {
            Some(Def::Struct {
                generics, methods, ..
            }) => {
                let (_, sig) = methods.iter().find(|(n, _)| n == method)?;
                Some((generics.clone(), sig.clone()))
            }
            _ => None,
        }
    }

    /// The key of a sorted container's `range` view, bound from `targs`: a
    /// struct declaring `range(r: Range<K>)` over its walk `range_each`, as
    /// `SortedBag<K>` does.
    pub(crate) fn sorted_view_key(&mut self, ty_name: &str, targs: &[Ty]) -> Option<Ty> {
        self.struct_method(ty_name, "range_each")?;
        let (generics, Ty::Fn { params, .. }) = self.struct_method(ty_name, "range")? else {
            return None;
        };
        let [p] = params.as_slice() else {
            return None;
        };
        let Ty::Named { name, args } = &p.ty else {
            return None;
        };
        let (true, [Ty::Param(k)]) = (name == "Range", args.as_slice()) else {
            return None;
        };
        let at = generics.iter().position(|g| g == k)?;
        targs.get(at).cloned()
    }

    /// True when `ty_name.method` takes a value of one of the struct's own
    /// type parameters, as `Deque<T>.push(x: T)` does: the container keeps it.
    pub(crate) fn method_takes_type_param(&mut self, ty_name: &str, method: &str) -> bool {
        let Some((generics, Ty::Fn { params, .. })) = self.struct_method(ty_name, method) else {
            return false;
        };
        params
            .iter()
            .any(|p| matches!(&p.ty, Ty::Param(g) if generics.contains(g)))
    }

    /// The fn `base.name` calls: the candidate it fits, else the only one.
    pub(super) fn method_def(&mut self, base: &Ty, name: &str) -> Option<Arc<Def>> {
        match self.pick_method(base, name) {
            Pick::One(_, def) => Some(def),
            _ => self.only_candidate(name),
        }
    }

    /// A member's hover type. A top-level fn met as a getter (`xs.first`),
    /// which lowering calls, shows as the fn the receiver picked.
    pub(super) fn shown_member(&mut self, ty: &Ty, name: &ast::Name) -> Ty {
        let picked = self.chosen_methods.iter().any(|r| r.0 == name.span);
        if *ty != Ty::Unknown || self.in_call_callee || !picked {
            return ty.clone();
        }
        match self.chosen_method(name).as_deref() {
            Some(Def::Fn { ty, .. }) => ty.clone(),
            _ => Ty::Unknown,
        }
    }

    /// The fn a call at `name` was resolved to, else the only candidate.
    pub(super) fn chosen_method(&mut self, name: &ast::Name) -> Option<Arc<Def>> {
        let at = (self.resolved.iter().rev())
            .chain(&self.chosen_methods)
            .find(|r| r.0 == name.span);
        let key = at.map(|(_, r)| match r.module.as_str() {
            "" => name.text.clone(),
            module => crate::prelude::method_key(&name.text, module),
        });
        match key
            .and_then(|k: String| self.global(&k))
            .filter(|d| takes_self(d))
        {
            Some(def) => Some(def),
            None => self.only_candidate(&name.text),
        }
    }

    /// The fn `x.name` calls when the receiver can't pick: the only
    /// candidate, never the first of several by declaration order.
    fn only_candidate(&mut self, name: &str) -> Option<Arc<Def>> {
        let mut candidates = self.method_candidates(name);
        (candidates.len() == 1).then(|| candidates.remove(0).1)
    }

    /// The top-level fn `name` whose `self` the struct, record or tuple
    /// `base` fits, as the method it is: the fn without that parameter.
    pub(super) fn own_method(&mut self, base: &Ty, name: &str) -> Option<Ty> {
        self.own_method_key(base, name).map(|(_, ty)| ty)
    }

    /// `own_method`, recording the choice at `name`.
    pub(super) fn own_method_at(&mut self, base: &Ty, name: &ast::Name) -> Option<Ty> {
        let (key, ty) = self.own_method_key(base, &name.text)?;
        self.record_method(name, &key);
        Some(ty)
    }

    fn own_method_key(&mut self, base: &Ty, name: &str) -> Option<(String, Ty)> {
        if !matches!(base, Ty::Named { .. } | Ty::Record(_) | Ty::Tuple(_)) {
            return None;
        }
        let Pick::One(key, def) = self.pick_method(base, name) else {
            return None;
        };
        let Def::Fn {
            ty: Ty::Fn { params, ret },
            generics,
            bounds,
            ..
        } = &*def
        else {
            return None;
        };
        // `w.last` on a `Two`: `S` is `Two`, and `T` what its `Bidir` gives.
        let mut bindings = crate::ty::Bindings::default();
        // Beside a built-in member of the name, the built-in answers.
        let builtin = super::call::catalog_owner(base)
            .is_some_and(|owner| crate::builtins::builtin_member(owner, name).is_some());
        if let Some(first) = params.first().filter(|_| !builtin) {
            first
                .ty
                .unify(&self.narrowed(base, &first.ty), &mut bindings);
        }
        self.bind_bound_params(generics, bounds, &mut bindings);
        let ty = Ty::Fn {
            params: params[1..].to_vec(),
            ret: ret.clone(),
        };
        Some((key, ty.subst(&bindings)))
    }

    /// Records which candidate `base.name` calls, when its receiver says.
    pub(super) fn note_method(&mut self, base: &Ty, name: &ast::Name) {
        // Beside a built-in member of the name, the call's arguments pick
        // (`method_form_call`): `xs.fold(+)` is the built-in's.
        let builtin = super::call::catalog_owner(base)
            .is_some_and(|owner| crate::builtins::builtin_member(owner, &name.text).is_some());
        if builtin {
            return;
        }
        if let Pick::One(key, _) = self.pick_method(base, &name.text) {
            self.record_method(name, &key);
        }
    }

    /// The `Resolved` at a method's name: the module of the fn it calls.
    fn record_method(&mut self, name: &ast::Name, key: &str) {
        let resolved = self.method_resolved(name, key);
        self.chosen_methods.retain(|r| r.0 != name.span);
        self.chosen_methods.push((name.span, resolved));
    }

    /// The `Resolved` of the method `key` a call at `name` could call.
    fn method_resolved(&mut self, name: &ast::Name, key: &str) -> Resolved {
        let module = if self.local_defs.contains_key(key) {
            None
        } else {
            self.env.origin(key)
        };
        Resolved {
            module: module.unwrap_or_default(),
            name: name.text.clone(),
            owner: None,
        }
    }

    /// The `Resolved` facts with each method choice not already among them.
    pub(super) fn with_chosen_methods(&mut self) -> Vec<(Span, Resolved)> {
        let mut resolved = std::mem::take(&mut self.resolved);
        for (span, chosen) in std::mem::take(&mut self.chosen_methods) {
            if !resolved.iter().any(|r| r.0 == span) {
                resolved.push((span, chosen));
            }
        }
        resolved
    }
}

impl<'a> Checker<'a> {
    /// `sorted(s)` for a prelude fold nothing else types: the same type as
    /// `s.sorted()`, whose element comes from `s` by the one `Seq` rule.
    pub(super) fn free_as_method(
        &mut self,
        call: &Expr,
        callee: &Expr,
        args: &[Arg],
    ) -> Option<Ty> {
        let ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        let (first, rest) = args.split_first()?;
        if first.name.is_some()
            || first.mutable
            || !crate::prelude::VALUE_NAMES.contains(&name.as_str())
            || self.local(name).is_some()
            || self.free_global(name).is_some()
        {
            return None;
        }
        self.quietly(|c| {
            let over = c.infer(&first.value);
            if over.is_vague() || c.element_ty(&over).is_vague() {
                return None;
            }
            let base = Box::new(first.value.clone());
            let name = cpl_syntax::ast::Name {
                text: name.clone(),
                span: callee.span,
            };
            let field = Expr {
                kind: ExprKind::Field { base, name },
                span: callee.span,
            };
            let kind = ExprKind::Call {
                callee: Box::new(field),
                args: rest.to_vec(),
            };
            let ty = c.infer(&Expr {
                kind,
                span: call.span,
            });
            let failed =
                (c.sink.diagnostics().iter()).any(|d| d.severity == cpl_diag::Severity::Error);
            (!failed && !ty.has_vague()).then_some(ty)
        })
    }

    /// The type of `base.name(args)` when `name` is a top-level fn taking
    /// `self`, found by checking the free form `name(base, args)`, when that
    /// checks cleanly to a plain value type. Its resolution, and its hover
    /// where the member has none, are the free form's in any case.
    pub(super) fn method_form_call(
        &mut self,
        call: &Expr,
        callee: &Expr,
        args: &[Arg],
    ) -> Option<Ty> {
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let module = matches!(&base.kind, ExprKind::Name(m) if self.local(m).is_none() && self.env.is_module(m));
        if module || self.local(&name.text).is_some() {
            return None;
        }
        // `Sub<V>.find(p)`: a type with `impl Index` is the `Seq` of its values (R04.2.20).
        if self.quietly(|c| c.library_domain_walk(base)).is_some() {
            return None;
        }
        let base_ty = self.infer_quietly(base);
        if name.text == "split" && base_ty == Ty::Str && self.split_on_str(args) {
            return Some(Ty::Error);
        }
        if self.declares_member(&base_ty, &name.text) {
            return None;
        }
        let pick = match self.pick_method_for(&base_ty, &name.text, Some(args)) {
            Pick::NoFit => self.pick_method(&base_ty, &name.text),
            pick => pick,
        };
        let key = match &pick {
            Pick::One(key, _) => key.clone(),
            Pick::Ambiguous(keys) => {
                self.ambiguous_candidates(base, name, keys);
                return Some(Ty::Error);
            }
            // `(m.get(0) ?? []).bsearch(f)` before the receiver is known: no
            // candidate is chosen by the order the fns were declared in.
            Pick::Unsettled
                if base_ty.is_vague() && self.method_candidates(&name.text).len() > 1 =>
            {
                return None;
            }
            Pick::Unsettled | Pick::NoFit => {
                self.method_candidates(&name.text).into_iter().next()?.0
            }
        };
        let own_member = match &base_ty {
            Ty::Optional(_) => crate::builtins::optional_member(&name.text),
            Ty::Domain(_) => {
                crate::builtins::is_member(crate::builtins::DOMAIN_MEMBERS, &name.text)
            }
            Ty::Pos(_) => crate::builtins::is_member(crate::builtins::POS_MEMBERS, &name.text),
            _ => self.catalog_row(base, &name.text).is_some(),
        };
        // A view has its source's members, `xs.rev.nth(1)` (R07.4.5).
        let vague = base_ty.has_vague()
            || matches!(base_ty, Ty::Fn { .. })
            || base_ty.view_source().is_some();
        let fits = own_member || vague || !matches!(pick, Pick::NoFit);
        let mut free_args = vec![Arg {
            name: None,
            mutable: false,
            value: (**base).clone(),
        }];
        free_args.extend(args.iter().cloned());
        let free = Expr {
            kind: ExprKind::Call {
                callee: Box::new(Expr {
                    kind: ExprKind::Name(key.clone()),
                    span: name.span,
                }),
                args: free_args,
            },
            span: call.span,
        };
        let (ty, hover, resolved, errors) = self.quietly(|c| {
            let ty = c.infer(&free);
            let errors: Vec<_> = (c.sink.diagnostics().iter())
                .filter(|d| d.severity == cpl_diag::Severity::Error)
                .cloned()
                .collect();
            let ty = if errors.is_empty() { ty } else { Ty::Unknown };
            let hover = c.hovers.iter().rev().find(|h| h.span == name.span).cloned();
            let resolved = c.resolved.iter().rev().find(|r| r.0 == name.span).cloned();
            (ty, hover, resolved, errors)
        });
        // No candidate takes the receiver for want of a bound: the free
        // form's E0606 is the method's, `?List<Int>` lacking the `Seq` that
        // `sum` requires. A receiver of another shape has a member of its
        // own, `(0..10).step(3)`, that the free form says nothing of.
        let free_failed = !errors.is_empty();
        let missing: Vec<_> = (errors.into_iter())
            .filter(|d| d.code == codes::MISSING_IMPL)
            .collect();
        // Only where the checker knows every trait the receiver has: an
        // optional, or a type the program or library declares. A built-in
        // container's members and traits are lowering's.
        let known = match &base_ty {
            Ty::Optional(_) => true,
            Ty::Named { name, .. } => (self.type_def(name))
                .is_some_and(|d| matches!(&*d, Def::Struct { .. } | Def::Enum { .. })),
            _ => false,
        };
        if !fits && known && !missing.is_empty() {
            for mut diag in missing {
                diag.message = diag
                    .message
                    .replace(&format!("`{key}`"), &format!("`{}`", name.text));
                self.sink.push(diag);
            }
            return Some(Ty::Error);
        }
        // A receiver no candidate takes calls none of them: an own member
        // such as `?T.map`, or a fn lowering finds by other means.
        if matches!(pick, Pick::NoFit) && free_failed {
            // Lowering may still find the fn, `(V, V).all(..)` over a
            // product: the name keeps the hover of the fn it would call.
            if let Some(mut hover) =
                hover.filter(|_| !own_member && !self.hovers.iter().any(|h| h.span == name.span))
            {
                hover.name = name.text.clone();
                self.hovers.push(hover);
            }
            return None;
        }
        // Arguments the fn doesn't take are the receiver's own member's,
        // `xs.fold(+)` beside `xs.fold_from(0, f)`.
        if own_member && free_failed {
            self.chosen_methods.retain(|r| r.0 != name.span);
            return None;
        }
        if let Some(mut hover) = hover.filter(|_| !self.hovers.iter().any(|h| h.span == name.span))
        {
            hover.name = name.text.clone();
            self.hovers.push(hover);
        }
        if let Some((span, mut resolved)) = resolved {
            resolved.name = name.text.clone();
            self.resolved.retain(|r| r.0 != span);
            self.resolved.push((span, resolved.clone()));
            // Kept past a probe that runs this call quietly, for lowering.
            self.chosen_methods.retain(|r| r.0 != span);
            self.chosen_methods.push((span, resolved));
        }
        // A built-in container's own member, `xs.sorted(..)` on a `List`, is
        // the catalog's to check; a library fn's container result is modelled.
        // A held `xs.windows(k)` is indexed by lowering's pieces (R07.7.9),
        // more than the stream its fn yields.
        let pieces = self.slices_of_self(&base_ty, &name.text);
        if plain_value(&ty)
            || self.declared_value(&ty)
            || (!own_member && !pieces && self.modelled(&ty))
        {
            // The probe's facts went with it, and the caller checks nothing
            // more: check the call for real so its arguments keep their
            // hovers, refs and warnings (a lambda's body).
            let (hovers, refs) = (self.hovers.len(), self.resolved.len());
            let calls = self.exprs.calls.len();
            self.infer(&free);
            // The call is the method's, `xs.sorted()`, not the free form's.
            let mut at = 0;
            self.exprs.calls.retain(|c| {
                at += 1;
                at <= calls || c.call != call.span
            });
            let mut at = 0;
            self.hovers.retain(|h| {
                at += 1;
                at <= hovers || h.span != name.span
            });
            let mut at = 0;
            self.resolved.retain(|r| {
                at += 1;
                at <= refs || r.0 != name.span
            });
            return Some(ty);
        }
        self.exprs.types.push((call.span, ty));
        None
    }
}

impl Checker<'_> {
    /// Whether the type `base` declares a fn `name` of its own, in its body,
    /// an impl or a trait it implements: that member, not a top-level fn
    /// of the name, is what `base.name` calls.
    pub(super) fn declares_member(&mut self, base: &Ty, name: &str) -> bool {
        let Ty::Named { name: owner, .. } = base else {
            return false;
        };
        self.member_traits(owner, name)
            .is_none_or(|t| !t.is_empty())
    }

    /// The traits whose impl for `owner` or own body gives it `name`, or `None` when
    /// `owner` declares `name` itself, in its body or an impl of no trait.
    pub(super) fn member_traits(&mut self, owner: &str, name: &str) -> Option<Vec<String>> {
        if let Some(Def::Struct { methods, .. } | Def::Enum { methods, .. }) =
            self.type_def(owner).as_deref()
        {
            if methods.iter().any(|(m, _)| m == name) {
                return None;
            }
        }
        let mut traits = Vec::new();
        for (trait_name, def) in self.env.impls(owner) {
            let in_impl = matches!(&*def, Def::Impl { methods, .. } if methods.iter().any(|(m, _)| m == name));
            let Some(t) = trait_name else {
                if in_impl {
                    return None;
                }
                continue;
            };
            let trait_def = self.type_def(t.split('<').next().unwrap_or(&t));
            let in_trait = matches!(trait_def.as_deref(), Some(Def::Trait { methods, .. })
                if methods.iter().any(|m| m.name == name));
            if (in_impl || in_trait) && !traits.contains(&t) {
                traits.push(t);
            }
        }
        Some(traits)
    }

    /// Whether `base.name(args)` gives arguments its built-in member's row
    /// doesn't take while a library fn fits the receiver: then the call is
    /// the library's, `xs.fold_from(0, f)` beside the built-in `xs.fold(+)`.
    /// `s.split(" ")`: a `Str` splits on a `Char` or a `Char -> Bool` (`Sep<T>`), so a one-char
    /// `Str` separator is fixed to its `Char`. True when it reported.
    fn split_on_str(&mut self, args: &[Arg]) -> bool {
        let [Arg {
            name: None, value, ..
        }] = args
        else {
            return false;
        };
        let ExprKind::Str(pieces) = &value.kind else {
            return false;
        };
        let [ast::StrPiece::Text(text)] = pieces.as_slice() else {
            return false;
        };
        let mut chars = text.chars();
        let (Some(ch), None) = (chars.next(), chars.next()) else {
            return false;
        };
        let lit = match ch {
            '\'' => "'\\''".to_string(),
            '\\' => "'\\\\'".to_string(),
            '\n' => "'\\n'".to_string(),
            '\t' => "'\\t'".to_string(),
            c => format!("'{c}'"),
        };
        if let Some(diag) = self.report(
            value.span,
            msgs::check::TYPE_MISMATCH_SPLIT_TAKES_A_CHAR,
            &[&self.quote(value.span)],
        ) {
            diag.help = Some(format!(
                "a `Str` splits on one `Char` or a `Char -> Bool`: `{lit}`"
            ));
            diag.fix(format!("write `{lit}`"), value.span, lit);
        }
        true
    }

    pub(super) fn past_builtin_row(&mut self, callee: &Expr, args: &[Arg]) -> bool {
        let ExprKind::Field { base, name } = &callee.kind else {
            return false;
        };
        let Some((row, _)) = self.catalog_row(base, &name.text) else {
            return false;
        };
        let base_ty = self.infer_quietly(base);
        let pick = match self.pick_method_for(&base_ty, &name.text, Some(args)) {
            Pick::NoFit => self.pick_method(&base_ty, &name.text),
            pick => pick,
        };
        let Pick::One(_, def) = pick else {
            return false;
        };
        let params = row_params(row.params);
        let takes_fn = |ty: &str| ty.contains("->") || (ty.len() == 1 && ty != "T");
        let positional = args.iter().filter(|a| a.name.is_none()).count();
        if positional > params.len() && takes_positionals(&def, positional) {
            self.exprs.past_rows.push(name.span);
            return true;
        }
        (args.iter())
            .any(|a| (a.name.as_ref()).is_some_and(|n| !params.iter().any(|(p, _)| *p == n.text)))
            || (args.iter().zip(&params)).any(|(a, (_, ty))| {
                matches!(a.value.kind, ExprKind::Lambda { .. }) && !takes_fn(ty)
            })
    }

    /// A named type a `for` walks is a `Seq`: `SortedRange<K>`, or a
    /// struct whose `each` yields its elements.
    pub(super) fn walks_by_each(&mut self, ty: &Ty) -> bool {
        matches!(ty, Ty::Named { .. }) && !matches!(self.element_ty(ty), Ty::Unknown | Ty::Error)
    }

    /// The type of `expr` for `Facts::type_of`. An untyped call keeps the
    /// library type `method_form_call` gave it.
    pub(super) fn record_shape(&mut self, expr: &Expr, ty: &Ty) {
        let kept = *ty == Ty::Unknown && self.exprs.types.last().is_some_and(|s| s.0 == expr.span);
        if !kept {
            self.exprs.types.push((expr.span, ty.clone()));
        }
    }
}

/// A type the checker models in full: no library type (`Slice<S>`, a
/// generator) whose members and traits only lowering knows, nothing vague.
fn plain_value(ty: &Ty) -> bool {
    match ty {
        Ty::Optional(inner) => plain_value(inner),
        Ty::Tuple(parts) => parts.iter().all(plain_value),
        Ty::Record(fields) => fields.iter().all(|(_, t)| plain_value(t)),
        Ty::Array { index, elem } => index.iter().all(plain_value) && plain_value(elem),
        Ty::Named { .. }
        | Ty::TypeName(_)
        | Ty::Fn { .. }
        | Ty::Param(_)
        | Ty::RowWithout(..)
        | Ty::Unknown
        | Ty::Error => false,
        _ => true,
    }
}

impl Checker<'_> {
    /// A struct the program or library declares, over plain values:
    /// `Prefix<I, Int>` from `b.scan(+)`.
    fn declared_value(&mut self, ty: &Ty) -> bool {
        let Ty::Named { name, args } = ty else {
            return false;
        };
        args.iter().all(|a| self.modelled(a))
            && (self.type_def(name)).is_some_and(|d| matches!(&*d, Def::Struct { .. }))
    }

    /// A type the checker models in full: a plain value, a declared struct,
    /// or a built-in container its catalog gives the members of, over such
    /// types: `Map<Int, List<Int>>` from `xs.group(by: f)`.
    fn modelled(&mut self, ty: &Ty) -> bool {
        if plain_value(ty) || self.declared_value(ty) {
            return true;
        }
        let Ty::Named { name, args } = ty else {
            return false;
        };
        // A `yields` fn's stream is a `Seq` of its element (R07.3.1), and so
        // is a range, `0..n` (R07.3.5).
        let catalogued = name == "Gen"
            || name == "Range"
            || (crate::builtins::BUILTIN_TYPES.iter())
                .any(|b| b.name == name && b.members.is_some());
        catalogued && args.iter().all(|a| self.modelled(a))
    }

    /// E0701 (R08.1.4): an extension fn with a `self` of its own shape that
    /// the receiver's own member always wins over is never called: `fn
    /// sum(self: Range<Int>)` beside `Range.sum`.
    pub(super) fn member_hides_extension(&mut self, callee: &Expr, args: &[Arg]) -> bool {
        let ExprKind::Field { base, name } = &callee.kind else {
            return false;
        };
        if self.method_candidates(&name.text).is_empty() {
            return false;
        }
        let base_ty = self.quietly(|c| c.infer(base));
        if base_ty.has_vague() {
            return false;
        }
        let own = self.declares_member(&base_ty, &name.text)
            || self.catalog_row(base, &name.text).is_some();
        if !own {
            return false;
        }
        let Pick::One(key, def) = self.pick_method_for(&base_ty, &name.text, Some(args)) else {
            return false;
        };
        let library = !self.local_defs.contains_key(&key)
            && (self.env.origin(&key)).is_some_and(|m| m == "prelude" || m.starts_with("std."));
        if library || self_ty(&def).is_none_or(|s| matches!(s, Ty::Param(_))) {
            return false;
        }
        let shown = self.quote(base.span);
        let method = &name.text;
        let ty = base_ty.to_string();
        let member = format!("{}.{method}", ty.split('<').next().unwrap_or(&ty));
        if let Some(diag) = self.report(
            name.span,
            msgs::check::AMBIGUOUS_METHOD_NEVER_CALLS_THE_FN,
            &[&shown, method, method, &member],
        ) {
            diag.help = Some(format!(
                "rename the fn, or call it as a plain fn: `{method}({shown})`"
            ));
        }
        true
    }

    /// E0701: two candidates fit the receiver equally well.
    fn ambiguous_candidates(&mut self, base: &Expr, name: &ast::Name, keys: &[String]) {
        let mut modules: Vec<String> = Vec::new();
        for key in keys {
            let origin = self.env.origin(key).unwrap_or_else(|| "this file".into());
            modules.push(format!("`{origin}`"));
        }
        let shown = self.quote(base.span);
        let method = &name.text;
        if let Some(diag) = self.report(
            name.span,
            msgs::check::AMBIGUOUS_METHOD_IS_AMBIGUOUS_EACH_GIVE,
            &[&shown, &method, &modules.join(" and "), &method],
        ) {
            diag.help = Some(format!(
                "import the one you mean and call it as `{method}({shown}, ..)`"
            ));
        }
        let candidates = keys.iter().map(|k| self.method_resolved(name, k)).collect();
        self.ambiguous.retain(|a| a.0 != name.span);
        self.ambiguous.push((name.span, candidates));
    }
}

/// A catalog row's parameters, `r: Range<T>, by: T -> K`, as names and types.
fn row_params(params: &str) -> Vec<(&str, &str)> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    for (i, c) in params.char_indices().chain([(params.len(), ',')]) {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                if let Some((n, t)) = params[start..i].split_once(':') {
                    out.push((n.trim(), t.trim()));
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    out
}

/// Whether each argument has the shape its parameter takes: a lambda goes
/// where a fn is taken, and a literal where a value is.
fn takes_shapes(def: &Def, args: &[Arg]) -> bool {
    let Def::Fn {
        ty: Ty::Fn { params, .. },
        ..
    } = def
    else {
        return true;
    };
    let after_self = params.get(1..).unwrap_or_default();
    let mut positional = after_self.iter().filter(|p| !p.named_only);
    args.iter().all(|arg| {
        let param = match &arg.name {
            Some(n) => after_self.iter().find(|p| p.name == n.text),
            None => positional.next(),
        };
        let Some(param) = param else {
            return true;
        };
        let lambda = matches!(arg.value.kind, ExprKind::Lambda { .. });
        let literal = matches!(
            arg.value.kind,
            ExprKind::Int(_)
                | ExprKind::Float(_)
                | ExprKind::Str(_)
                | ExprKind::Char(_)
                | ExprKind::Bool(_)
        );
        match &param.ty {
            Ty::Fn { .. } => !literal,
            Ty::Param(_) | Ty::Unknown => true,
            _ => !lambda,
        }
    })
}

impl Members {
    /// Method `method`'s type on `on`: `bindings`, plus what `on` binds the
    /// method's own type parameters to when it writes its receiver's type
    /// (`self: Rev<[D]T>` called on a `Rev<List<Int>>`).
    pub(super) fn member_ty(
        &self,
        method: &str,
        ty: &Ty,
        on: &Ty,
        bindings: &crate::ty::Bindings,
    ) -> Ty {
        let mut bindings = bindings.clone();
        if let Some((_, takes)) = self.selves.iter().find(|(m, _)| m == method) {
            takes.unify(on, &mut bindings);
        }
        ty.subst(&bindings)
    }
}

#[derive(Default)]
pub(crate) struct Members {
    pub(crate) generics: Vec<String>,
    pub(crate) fields: Vec<crate::ty::Param>,
    pub(crate) members: Vec<(String, Ty)>,
    pub(crate) projections: Vec<crate::def::Projection>,
    /// The methods that take `mut self`.
    pub(crate) mutators: Vec<String>,
    pub(crate) selves: Vec<(String, Ty)>,
}

impl Members {
    /// What each type parameter is for a value of the type with `args`.
    pub(crate) fn bindings(&self, args: &[Ty]) -> crate::ty::Bindings {
        if args.len() != self.generics.len() {
            return crate::ty::Bindings::default();
        }
        self.generics
            .iter()
            .cloned()
            .zip(args.iter().cloned())
            .collect()
    }
}

/// True when a type parameter (`D`) is left in `ty`: it was not worked out.
fn names_param(ty: &Ty) -> bool {
    let mut found = false;
    ty.map(&mut |t| {
        found |= matches!(t, Ty::Param(_));
        found.then(|| t.clone())
    });
    found
}

/// Whether the method `def` takes `given` positional arguments after `self`.
/// R06.1.6: the labels a call must give `def`, sorted.
pub(super) fn def_labels(def: &Def) -> Vec<&str> {
    let Def::Fn {
        ty: Ty::Fn { params, .. },
        ..
    } = def
    else {
        return Vec::new();
    };
    let mut out: Vec<&str> = (params.iter())
        .filter(|p| p.named_only && !p.has_default && !p.variadic)
        .map(|p| p.name.as_str())
        .collect();
    out.sort_unstable();
    out
}

/// Whether the call's labels fit `def`: each it requires, and only ones it has.
pub(super) fn def_labels_fit(def: &Def, args: &[Arg]) -> bool {
    let Def::Fn {
        ty: Ty::Fn { params, .. },
        ..
    } = def
    else {
        return true;
    };
    let given: Vec<&str> = args
        .iter()
        .filter_map(|a| a.name.as_ref().map(|n| n.text.as_str()))
        .collect();
    def_labels(def).iter().all(|r| given.contains(r))
        && given.iter().all(|g| params.iter().any(|p| p.name == *g))
}

fn takes_count(def: &Def, given: usize) -> bool {
    let Def::Fn {
        ty: Ty::Fn { params, .. },
        ..
    } = def
    else {
        return false;
    };
    let after: Vec<_> = params.iter().skip(1).filter(|p| !p.named_only).collect();
    let required = after.iter().filter(|p| !p.has_default).count();
    after.iter().any(|p| p.variadic) || (required <= given && given <= after.len())
}
