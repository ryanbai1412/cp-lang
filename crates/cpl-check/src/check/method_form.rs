//! `xs.f(args)` on a library `fn f(self: S, ...)` checks as `f(xs, args)`.
//! R11.3.4: every top-level `fn f(self: ..)` in reach, the file's own and
//! each module's, is a candidate, and the call takes the one whose `self`
//! its receiver fits. The choice is the `Resolved` at the method's name,
//! which lowering reads.

use super::*;
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

/// The members an optional has of its own, whatever std declares.
pub(super) const OPTIONAL_MEMBERS: &[&str] = &["map", "filter"];

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

/// R08.1.4: `a` is more specific than `b` when `b`'s `self` binds to
/// `a`'s and not the other way. A bare `self: S` is no shape, so it is
/// never more or less specific than another.
fn more_specific(a: &Option<Ty>, b: &Option<Ty>) -> bool {
    let (Some(a), Some(b)) = (a, b) else {
        return false;
    };
    !matches!(a, Ty::Param(_))
        && !matches!(b, Ty::Param(_))
        && instance_of(a, b)
        && !instance_of(b, a)
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
        let Some(this) = self.global(&mine).map(|d| self_ty(&d)) else {
            return;
        };
        for key in own.iter().filter(|k| **k != mine) {
            if !at(self, key).is_some_and(|i| i < here) {
                continue;
            }
            let Some(that) = self.global(key).map(|d| self_ty(&d)) else {
                continue;
            };
            if !more_specific(&this, &that) && !more_specific(&that, &this) {
                self.error(
                    codes::OVERLOAD_BY_TYPE,
                    f.name.span,
                    format!("a second `fn {name}` whose `self` is no more and no less specific than the first one's: fns are not overloaded by their other parameters"),
                );
                return;
            }
        }
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
        let base_ty = self.quietly(|c| c.infer(base));
        let Pick::One(_, def) = self.pick_method(&base_ty, &name.text) else {
            return None;
        };
        if !matches!(&*def, Def::Fn { getter: true, .. }) {
            return None;
        }
        // A call giving a value of a struct leaves its type in `shapes`.
        let read = |c: &mut Self| {
            let ty = c.method_form_call(expr, expr, &[]);
            ty.or_else(|| {
                let shape = c.shapes.iter().rev().find(|(at, _)| *at == expr.span);
                shape.map(|(_, ty)| ty.clone())
            })
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
        first.ty.unify(base, &mut bindings);
        let takes = first.ty.subst(&bindings);
        // `self: Range<F64>` has `Range<Int>`'s shape but doesn't take it.
        let named = matches!(first.ty, Ty::Named { .. });
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
            if self.self_fits(base, &def) && args.is_none_or(|args| takes_shapes(&def, args)) {
                fit.push((key, def));
            }
        }
        // R11.2.3: a name the file declares or a `use` imports shadows the
        // prelude's, so the prelude's candidates drop out beside one.
        let shadows: Vec<bool> = fit
            .iter()
            .map(|f| self.env.origin(&f.0).is_none() || self.env.chosen(&f.0))
            .collect();
        if shadows.contains(&true) {
            let mut shadows = shadows.into_iter();
            fit.retain(|_| shadows.next().unwrap_or(false));
        }
        // R08.1.4: of the candidates, those no other is more specific than.
        let selfs: Vec<Option<Ty>> = fit.iter().map(|f| self_ty(&f.1)).collect();
        let beaten: Vec<bool> = (0..fit.len())
            .map(|b| (0..fit.len()).any(|a| more_specific(&selfs[a], &selfs[b])))
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
        match fit.len() {
            0 => Pick::NoFit,
            1 => {
                let (key, def) = fit.pop().unwrap();
                Pick::One(key, def)
            }
            _ => Pick::Ambiguous(fit.into_iter().map(|f| f.0).collect()),
        }
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
            first.ty.unify(base, &mut bindings);
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
        let module = if self.local_defs.contains_key(key) {
            None
        } else {
            self.env.origin(key)
        };
        let resolved = Resolved {
            module: module.unwrap_or_default(),
            name: name.text.clone(),
            owner: None,
        };
        self.chosen_methods.retain(|r| r.0 != name.span);
        self.chosen_methods.push((name.span, resolved));
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
        let base_ty = self.quietly(|c| c.infer(base));
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
            Ty::Optional(_) => OPTIONAL_MEMBERS.contains(&name.text.as_str()),
            Ty::Domain(_) => crate::builtins::DOMAIN_MEMBERS.contains(&name.text.as_str()),
            Ty::Pos(_) => crate::builtins::POS_MEMBERS.contains(&name.text.as_str()),
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
        if plain_value(&ty) || self.declared_value(&ty) || (!own_member && self.modelled(&ty)) {
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
        self.shapes.push((call.span, ty));
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
        if let Some(Def::Struct { methods, .. } | Def::Enum { methods, .. }) =
            self.type_def(owner).as_deref()
        {
            if methods.iter().any(|(m, _)| m == name) {
                return true;
            }
        }
        for (trait_name, def) in self.env.impls(owner) {
            if matches!(&*def, Def::Impl { methods, .. } if methods.iter().any(|(m, _)| m == name))
            {
                return true;
            }
            let trait_def = (trait_name.as_deref())
                .and_then(|t| t.split('<').next())
                .and_then(|t| self.type_def(t));
            if matches!(trait_def.as_deref(), Some(Def::Trait { methods, .. })
                if methods.iter().any(|m| m.name == name))
            {
                return true;
            }
        }
        false
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
        let message = format!(
            "`split` takes a `Char` separator, but `{}` is a `Str`",
            self.quote(value.span)
        );
        if let Some(diag) = self.error(codes::TYPE_MISMATCH, value.span, message) {
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
        let base_ty = self.quietly(|c| c.infer(base));
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

    /// The type of `expr` for `Facts::type_of`, and of a call, index or
    /// other compound expression for the editor's `.`. An untyped call
    /// keeps the library type `method_form_call` gave it.
    pub(super) fn record_shape(&mut self, expr: &Expr, ty: &Ty) {
        self.exprs.types.push((expr.span, ty.clone()));
        let compound = matches!(
            expr.kind,
            ExprKind::Call { .. }
                | ExprKind::Index { .. }
                | ExprKind::Paren(_)
                | ExprKind::Range { .. }
                | ExprKind::Unwrap(_)
                | ExprKind::Try(_)
        );
        let kept = *ty == Ty::Unknown && self.shapes.last().is_some_and(|s| s.0 == expr.span);
        if compound && !kept {
            self.shapes.push((expr.span, ty.clone()));
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
        let catalogued =
            (crate::builtins::BUILTIN_TYPES.iter()).any(|b| b.name == name && b.members.is_some());
        catalogued && args.iter().all(|a| self.modelled(a))
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
        let message = format!(
            "`{shown}.{method}` is ambiguous: {} each give its type a `{method}`",
            modules.join(" and ")
        );
        if let Some(diag) = self.error(codes::AMBIGUOUS_METHOD, name.span, message) {
            diag.help = Some(format!(
                "import the one you mean and call it as `{method}({shown}, ..)`"
            ));
        }
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
