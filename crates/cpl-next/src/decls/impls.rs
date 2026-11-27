//! Which impl a type has of a trait, and which member a receiver reaches through one.
use super::*;

impl Decls {
    pub fn lookup_member(&self, recv: &Ty, name: &str) -> Option<Member> {
        if let Ty::Struct(d, targs, sets) = recv {
            let s = self.s(*d);
            if let Some(i) = s.fields.iter().position(|(n, _)| n == name) {
                let sub = Subst { tys: targs, sets };
                return Some(Member::Field(i, sub.ty(&s.fields[i].1)));
            }
            if let Some(m) = s.methods.get(name) {
                return Some(Member::Method(*m, targs.clone(), sets.clone()));
            }
        }
        self.impl_member(recv, name, None)
    }
    /// `r` with each parameter it leaves out given its default, for `self_ty` as `Self` (R08.1.1).
    pub(super) fn with_defaults(&self, mut r: TraitRef, self_ty: &Ty) -> TraitRef {
        let defaults = &self.traits[r.tr.0].defaults;
        while let Some(Some(t)) = defaults.get(r.args.len()) {
            let mut tys = vec![self_ty.clone()];
            tys.extend(r.args.iter().cloned());
            let t = Subst {
                tys: &tys,
                sets: &[],
            }
            .ty(t);
            r.args.push(t);
        }
        r
    }
    /// The fn `name` of an impl for `recv`, of the trait named `trait_name` when one is given,
    /// or the trait's default when the impl does not write it (R08.1.2).
    pub fn impl_member(&self, recv: &Ty, name: &str, trait_name: Option<&str>) -> Option<Member> {
        self.impl_member_with(recv, name, trait_name, |_| true)
    }
    /// The fn `name` of an impl for `recv` whose declaration `pick` takes: the `at` that reads
    /// (`Indexed`) or the one that lends a place (`IndexedMut`), told apart by signature.
    pub fn impl_member_by(
        &self,
        recv: &Ty,
        name: &str,
        pick: impl Fn(&FnDecl) -> bool,
    ) -> Option<Member> {
        self.impl_member_with(recv, name, None, pick)
    }
    pub(super) fn impl_member_with(
        &self,
        recv: &Ty,
        name: &str,
        trait_name: Option<&str>,
        pick: impl Fn(&FnDecl) -> bool,
    ) -> Option<Member> {
        let sets = match recv {
            Ty::Struct(_, _, ss) => ss.clone(),
            _ => Vec::new(),
        };
        // A value's member has a `self`: `Domain.at(x)` is not `Int`'s `at`.
        let pick = |f: &FnDecl| f.self_mode != SelfMode::None && pick(f);
        let ordered = (self.impls.iter().filter(|im| !im.blanket()))
            .chain(self.impls.iter().filter(|im| im.blanket()));
        let found = ordered.into_iter().find_map(|im| {
            let tr = &self.traits[im.trait_.tr.0];
            if trait_name.is_some_and(|n| n != tr.name) {
                return None;
            }
            if let Some(m) = im.methods.get(name).filter(|m| pick(self.f(**m))) {
                let targs = self.fits_impl(im, recv)?;
                return Some(Member::Method(*m, targs, sets.clone()));
            }
            let tm = *tr.methods.get(name)?;
            if matches!(self.f(tm).body, FnBody::None) || !pick(self.f(tm)) {
                return None;
            }
            let binds = self.fits_impl(im, recv)?;
            let sub = Subst {
                tys: &binds,
                sets: &[],
            };
            let mut targs = vec![recv.clone()];
            targs.extend(im.trait_.args.iter().map(|t| sub.ty(t)));
            Some(Member::Method(tm, targs, Vec::new()))
        });
        // A declared domain meets `Domain` with no impl: its members are the trait's defaults.
        found.or_else(|| {
            let tr = self.domain_trait.filter(|_| matches!(recv, Ty::Dom(_)))?;
            let tr = &self.traits[tr.0];
            let tm = *tr
                .methods
                .get(name)
                .filter(|_| trait_name.is_none_or(|n| n == tr.name))?;
            pick(self.f(tm)).then(|| Member::Method(tm, vec![recv.clone()], Vec::new()))
        })
    }
    /// Whether every field of `s` has an impl of `tr`, as a derived impl needs (R08.1.6).
    pub(super) fn fields_have(&self, s: StructId, tr: TraitId) -> bool {
        let sd = self.s(s);
        (sd.fields.iter().zip(&sd.cached)).all(|((_, t), cached)| *cached || self.has_impl(t, tr))
    }
    /// A trait made only of bodiless `@get` members, met by fields of those names (R12.11.3).
    pub(super) fn field_trait(&self, tr: TraitId) -> bool {
        let t = &self.traits[tr.0];
        let bodiless = |m: &FnId| self.f(*m).getter && matches!(self.f(*m).body, ast::FnBody::None);
        !t.alias && !t.methods.is_empty() && t.methods.values().all(bodiless)
    }
    /// The impls of `tr`, those for a written type before blanket ones (R08.1.2).
    pub fn impls_of(&self, tr: TraitId) -> impl Iterator<Item = &ImplDecl> {
        let of = move |b: bool| {
            (self.impls.iter()).filter(move |im| im.trait_.tr == tr && im.blanket() == b)
        };
        of(false).chain(of(true))
    }
    /// The impl's type arguments for `t`; a blanket impl only where `t` meets its bounds.
    pub fn fits_impl(&self, im: &ImplDecl, t: &Ty) -> Option<Vec<Ty>> {
        let targs = self.impl_args(im, t)?;
        (!im.blanket() || self.bounds_met(im, &targs)).then_some(targs)
    }
    fn bounds_met(&self, im: &ImplDecl, targs: &[Ty]) -> bool {
        (im.bounds.iter()).all(|(i, b)| targs.get(*i).is_none_or(|a| self.has_impl(a, b.tr)))
    }
    /// Whether `t` has an impl of `tr`, its parameters taken to meet the impl's bounds.
    pub fn has_impl(&self, t: &Ty, tr: TraitId) -> bool {
        if matches!(t, Ty::Param(_) | Ty::Var(_) | Ty::Error) {
            return true;
        }
        if Some(tr) == self.domain_trait && matches!(t, Ty::Dom(_)) {
            return true;
        }
        (self.impls_of(tr)).any(|im| {
            self.impl_args(im, t)
                .is_some_and(|targs| self.bounds_met(im, &targs))
        })
    }
    /// The library method `name` whose `self` can be `recv` (R11.3.4).
    pub fn method_for(&self, name: &str, recv: &Ty) -> Option<FnId> {
        (self.methods.get(name)?.iter().copied()).find(|f| self.self_fits(*f, recv))
    }
    /// Whether `recv` has the shape of `f`'s `self`: the same struct, or the same kind of type.
    pub fn self_fits(&self, f: FnId, recv: &Ty) -> bool {
        match (self.f(f).self_ty.as_ref(), recv) {
            (Some(Ty::Param(_)), _) | (_, Ty::Var(_) | Ty::Error) => true,
            (Some(Ty::Struct(a, ..)), Ty::Struct(b, ..)) => a == b,
            (Some(t), r) => std::mem::discriminant(t) == std::mem::discriminant(r),
            (None, _) => false,
        }
    }
    /// The impl's type arguments when it is for `t`; a type not known yet fits any.
    pub fn impl_args(&self, im: &ImplDecl, t: &Ty) -> Option<Vec<Ty>> {
        self.impl_args_for(im, t, &[])
    }
    /// The impl's type arguments when it is for `t` and of the trait with arguments `trait_args`
    /// (`impl Into<Char> for Int` and not `impl Into<U64> for Int`); fewer are not checked.
    pub fn impl_args_for(&self, im: &ImplDecl, t: &Ty, trait_args: &[Ty]) -> Option<Vec<Ty>> {
        // Two types that may yet be one: `Optional<Int>` and `Optional<_>`.
        fn agree(a: &Ty, b: &Ty) -> bool {
            match (a, b) {
                (Ty::Var(_) | Ty::Error, _) | (_, Ty::Var(_) | Ty::Error) => true,
                (Ty::Raw(a), Ty::Raw(b)) | (Ty::Ref(a), Ty::Ref(b)) => agree(a, b),
                (Ty::Struct(d1, a, _), Ty::Struct(d2, b, _)) => {
                    d1 == d2 && a.len() == b.len() && a.iter().zip(b).all(|(x, y)| agree(x, y))
                }
                _ => a == b,
            }
        }
        fn fit(p: &Ty, t: &Ty, binds: &mut [Option<Ty>]) -> bool {
            match (p, t) {
                // A parameter binds even to a type not known yet, so the method's own
                // arguments follow it once it is.
                (Ty::Param(i), _) => match &binds[*i] {
                    Some(b) => agree(b, t),
                    None if *t == Ty::Error => true,
                    None => {
                        binds[*i] = Some(t.clone());
                        true
                    }
                },
                (_, Ty::Var(_) | Ty::Error) => true,
                (Ty::Raw(a), Ty::Raw(b)) | (Ty::Ref(a), Ty::Ref(b)) => fit(a, b, binds),
                (Ty::Struct(d1, a, _), Ty::Struct(d2, b, _)) => {
                    d1 == d2 && a.len() == b.len() && a.iter().zip(b).all(|(x, y)| fit(x, y, binds))
                }
                (Ty::Range(..) | Ty::RangeOf(_), Ty::Int)
                | (Ty::Int, Ty::Range(..) | Ty::RangeOf(_)) => true,
                _ => p == t,
            }
        }
        let mut binds = vec![None; im.generics.len()];
        if !fit(&im.for_, t, &mut binds)
            || !(self.traits[im.trait_.tr.0].inputs(&im.trait_.args).iter())
                .zip(self.traits[im.trait_.tr.0].inputs(trait_args))
                .all(|(p, a)| fit(p, a, &mut binds))
        {
            return None;
        }
        // A parameter only a bound names (`D` of `Mapped<S: Indexed<D, T>, T, U>`) is what
        // the impl meeting that bound fixes it to (R08.1.2).
        loop {
            let unbound = binds.iter().filter(|b| b.is_none()).count();
            for (i, r) in &im.bounds {
                let Some(bt) = binds[*i].clone() else {
                    continue;
                };
                let met = self.impls.iter().filter(|m| m.trait_.tr == r.tr);
                let Some(args) = met.into_iter().find_map(|m| {
                    let b = self.impl_args(m, &bt)?;
                    let sub = Subst { tys: &b, sets: &[] };
                    Some(m.trait_.args.iter().map(|a| sub.ty(a)).collect::<Vec<_>>())
                }) else {
                    continue;
                };
                for (p, a) in r.args.iter().zip(&args) {
                    fit(p, a, &mut binds);
                }
            }
            if unbound == 0 || binds.iter().filter(|b| b.is_none()).count() == unbound {
                break;
            }
        }
        Some(binds.into_iter().map(|b| b.unwrap_or(Ty::Error)).collect())
    }
}

/// An impl of a trait for a type needs one of each of its supertraits for that type (R08.1.1).
/// The library's are checked with it; here one the slice refuses would read as missing.
pub(super) fn supertraits_unmet(d: &Decls) -> Vec<(usize, &'static str, String)> {
    let mut out = Vec::new();
    for im in d.impls.iter().filter(|im| !d.is_library(im.module)) {
        let tr = &d.traits[im.trait_.tr.0];
        for sup in &tr.supers {
            let met = match &im.for_ {
                Ty::Param(i) => im.bounds.iter().any(|(j, r)| j == i && r.tr == sup.tr),
                t => (d.impls.iter()).any(|o| o.trait_.tr == sup.tr && d.impl_args(o, t).is_some()),
            };
            if !met {
                out.push((
                    im.at,
                    cpl_diag::codes::MISSING_IMPL.0,
                    format!(
                        "`impl {}` needs an `impl {}` for the same type (R08.1.1)",
                        tr.name, d.traits[sup.tr.0].name
                    ),
                ));
            }
        }
        // Its choice for an associated type has the traits the trait's bound names (R08.1.1).
        for (a, ab) in &tr.assoc_bounds {
            let Some(t) = im.trait_.args.get(a - 1) else {
                continue;
            };
            let met = match t {
                Ty::Param(i) => im.bounds.iter().any(|(j, r)| j == i && r.tr == ab.tr),
                Ty::Error => true,
                t => d.has_impl(t, ab.tr),
            };
            if !met {
                let k = a - (tr.defaults.len() + 1 - tr.assoc.len());
                out.push((
                    im.assoc_at.get(k).copied().unwrap_or(im.at),
                    cpl_diag::codes::MISSING_IMPL.0,
                    format!(
                        "`impl {}` chooses a type for `{}` with no `impl {}` (R08.1.1)",
                        tr.name, tr.assoc[k], d.traits[ab.tr.0].name
                    ),
                ));
            }
        }
    }
    out
}
/// Whether one type could fit both impl headers. A bare parameter and a written type don't: the
/// impl for the written type is more specific and is the one chosen (R08.1.2).
pub(super) fn overlap(a: &Ty, b: &Ty) -> bool {
    match (a, b) {
        (Ty::Param(_), Ty::Param(_)) => true,
        (Ty::Param(_), _) | (_, Ty::Param(_)) => false,
        _ => same_shape(a, b),
    }
}
/// Whether one type could be both, a parameter fitting anything.
pub(super) fn same_shape(a: &Ty, b: &Ty) -> bool {
    match (a, b) {
        (Ty::Param(_), _) | (_, Ty::Param(_)) => true,
        (Ty::Raw(x), Ty::Raw(y)) | (Ty::Ref(x), Ty::Ref(y)) => same_shape(x, y),
        (Ty::Struct(d1, x, _), Ty::Struct(d2, y, _)) => {
            d1 == d2 && x.iter().zip(y).all(|(x, y)| same_shape(x, y))
        }
        _ => a == b,
    }
}
