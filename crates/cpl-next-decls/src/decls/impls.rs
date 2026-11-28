//! Which impl a type has of a trait, and which member a receiver reaches through one.
use super::*;
use crate::views::{Items, Methods, Signatures};

/// Which impl a type takes when several fit: one for its written type (0) before one for
/// every record with some fields (1), before any other blanket one (2) (R08.1.2, R08.4.5).
pub fn impl_rank(items: &dyn Items, im: &ImplDecl) -> u8 {
    rank_of(items, &im.for_, &im.bounds)
}
/// `impl_rank` of an impl for `for_` with `bounds`.
pub fn rank_of(items: &dyn Items, for_: &Ty, bounds: &[(usize, TraitRef)]) -> u8 {
    let Ty::Param(i) = *for_ else { return 0 };
    match (bounds.iter()).any(|(k, b)| *k == i && items.t(b.tr).row) {
        true => 1,
        false => 2,
    }
}
/// Which of struct `s`'s fields is named `name`.
fn field(items: &dyn Items, s: StructId, name: Sym) -> Option<usize> {
    (items.s(s).fields.iter()).position(|(f, _)| *f == name)
}

/// An impl's place in one lookup: decls' impl `i`, or row `i` of the layer checking a body. It
/// keys one answer, never a stored table.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ImplIdx {
    Decl(usize),
    Row(usize),
}

impl Decls {
    /// `r` with each parameter it leaves out given its default, for `self_ty` as `Self` (R08.1.1).
    pub fn with_defaults(&self, mut r: TraitRef, self_ty: &Ty) -> TraitRef {
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
    /// A trait made only of bodiless `@get` members, met by fields of those names (R12.11.3).
    pub(super) fn field_trait(&self, tr: TraitId) -> bool {
        let t = &self.traits[tr.0];
        let bodiless = |m: &FnId| self.f(*m).getter && self.f(*m).bodiless();
        !t.alias && !t.methods.is_empty() && t.methods.values().all(bodiless)
    }
    /// The impls every domain has, `Domain` and `Ordered`, for domain `dom` declared at `at`:
    /// they write no members, so each member is the trait's default (R04.2, R04.2.31).
    pub fn domain_rows(&self, dom: usize, at: SourceId, module: ModId) -> Vec<ImplDecl> {
        let traits = [self.roles.domain, self.roles.order];
        (traits.into_iter().flatten())
            .map(|tr| ImplDecl {
                trait_: TraitRef {
                    tr,
                    args: Vec::new(),
                },
                generics: Vec::new(),
                bounds: Vec::new(),
                for_: Ty::Dom(dom),
                methods: HashMap::new(),
                at: at.clone(),
                assoc_at: Vec::new(),
                module,
                derived: false,
                replaced: false,
            })
            .collect()
    }
    /// The rows of each domain decls declared since the last call (`domain_rows`).
    fn add_domain_rows(&mut self) {
        let mut doms: Vec<(usize, ItemId)> = (self.dom_ids.iter())
            .map(|((item, _), &i)| (i, item.clone()))
            .collect();
        doms.sort_by_key(|(i, _)| *i);
        for (i, item) in doms.into_iter().skip(self.dom_rows_made) {
            let module = (self.module_names.iter())
                .position(|m| **m == *item.module)
                .map_or(self.program, ModId);
            for im in self.domain_rows(i, item.at(Span::default()), module) {
                self.push_impl(im);
            }
        }
        self.dom_rows_made = self.dom_ids.len();
    }
    /// Fills `impls_ranked` from the impls a lookup may find now, each domain's rows with them.
    pub fn rank_impls(&mut self) {
        self.add_domain_rows();
        let rank = |ks: &[usize]| {
            let mut ks: Vec<usize> = (ks.iter().copied())
                .filter(|&k| self.live(&self.impls[k]))
                .collect();
            ks.sort_by_key(|&k| self.impl_rank(&self.impls[k]));
            ks
        };
        let by_trait = (self.impls_by_trait.iter())
            .map(|(tr, ks)| (*tr, rank(ks)))
            .collect();
        self.impls_ranked = Some(by_trait);
    }
    /// The library method `name` whose `self` can be `recv` ([`method_for`]).
    pub fn method_for(&self, name: &str, recv: &Ty) -> Option<FnId> {
        method_for(self, self, self.sym(name)?, recv)
    }
    /// [`shaped_member`], by text.
    pub fn shaped_member(&self, recv: &Ty, name: &str) -> Option<FnId> {
        shaped_member(self, self, recv, self.sym(name)?)
    }
    pub fn self_fits(&self, f: FnId, recv: &Ty) -> bool {
        self_fits(self, f, recv)
    }
}

/// The one impl lookup (R08.1.2): decls' impls, then the rows the layer checking a body adds,
/// the `Domain` and `Ordered` impls of each domain it declares.
#[derive(Clone, Copy)]
pub struct Impls<'a> {
    pub items: &'a dyn Items,
    pub rows: &'a [ImplDecl],
}

impl<'a> Impls<'a> {
    /// Decls' impls alone.
    pub fn of(items: &'a dyn Items) -> Impls<'a> {
        Impls { items, rows: &[] }
    }
    /// The impls of `tr` a lookup may find, decls' in order and then the rows.
    pub fn trait_impls(
        &self,
        tr: TraitId,
    ) -> impl Iterator<Item = (ImplIdx, &'a ImplDecl)> + Clone {
        let items = self.items;
        let rows = (self.rows.iter().enumerate())
            .filter(move |(_, im)| im.trait_.tr == tr)
            .map(|(j, im)| (ImplIdx::Row(j), im));
        (items.impls_by_trait(tr).iter())
            .filter(move |&&k| items.live(k))
            .map(move |&k| (ImplIdx::Decl(k), items.imp(k)))
            .chain(rows)
    }
    /// Impl `k`: decls' by index, then the rows.
    pub fn imp(&self, k: ImplIdx) -> &'a ImplDecl {
        match k {
            ImplIdx::Decl(i) => self.items.imp(i),
            ImplIdx::Row(j) => &self.rows[j],
        }
    }
    pub fn lookup_member(&self, recv: &Ty, name: Sym) -> Option<Member> {
        if let Ty::Struct(d, targs, sets) = recv {
            let s = self.items.s(*d);
            if let Some(i) = field(self.items, *d, name) {
                let sub = Subst { tys: targs, sets };
                return Some(Member::Field(i, sub.ty(&s.fields[i].1)));
            }
            if let Some(m) = s.methods.get(&name).copied() {
                return Some(Member::Method(m, targs.clone(), sets.clone()));
            }
        }
        self.impl_member(recv, name, None)
    }
    /// The fn `name` of an impl for `recv`, of trait `trait_` when one is given,
    /// or the trait's default when the impl does not write it (R08.1.2).
    pub fn impl_member(&self, recv: &Ty, name: Sym, trait_: Option<TraitId>) -> Option<Member> {
        self.impl_member_with(recv, name, trait_, |_| true)
    }
    /// The fn `name` of an impl for `recv` whose declaration `pick` takes: the `at` that reads
    /// (`Indexed`) or the one that lends a place (`IndexedMut`), told apart by signature.
    pub fn impl_member_by(
        &self,
        recv: &Ty,
        name: Sym,
        pick: impl Fn(&FnDecl) -> bool,
    ) -> Option<Member> {
        self.impl_member_with(recv, name, None, pick)
    }
    pub(super) fn impl_member_with(
        &self,
        recv: &Ty,
        name: Sym,
        trait_: Option<TraitId>,
        pick: impl Fn(&FnDecl) -> bool,
    ) -> Option<Member> {
        let mut choices = self.impl_choices(recv, name, trait_, &pick);
        if choices.len() > 1 {
            return None;
        }
        choices.pop().map(|(_, m)| m)
    }
    /// Each fn `name` an impl for `recv` gives, of trait `trait_` when one is given,
    /// with the trait the impl is of: the fitting impls of the most specific rank, so an impl
    /// for a written type beats a blanket one of another trait (R08.1.4). More than one is no
    /// choice: the call says which trait or arguments it means.
    pub fn impl_choices(
        &self,
        recv: &Ty,
        name: Sym,
        trait_: Option<TraitId>,
        pick: impl Fn(&FnDecl) -> bool,
    ) -> Vec<(TraitRef, Member)> {
        // A value's member has a `self`: `Domain.at(x)` is not `Int`'s `at`.
        let pick = |f: &FnDecl| f.self_mode != SelfMode::None && pick(f);
        let live = (self.items.impls_named(name).iter())
            .filter(|&&k| self.items.live(k))
            .map(|&k| self.items.imp(k))
            .chain(
                (self.rows.iter())
                    .filter(|im| (self.items.t(im.trait_.tr).methods).contains_key(&name)),
            );
        let mut found: Vec<(u8, TraitRef, Member)> = Vec::new();
        for im in live {
            let sets = match recv {
                Ty::Struct(_, _, ss) => ss.clone(),
                _ => {
                    let mut sets = Vec::new();
                    head_sets(&im.for_, recv, &mut sets);
                    sets
                }
            };
            let Some(member) = self.impl_gives(im, recv, name, trait_, &pick, &sets) else {
                continue;
            };
            let rank = impl_rank(self.items, im);
            match found.first().map(|f| f.0) {
                Some(best) if best < rank => continue,
                Some(best) if best > rank => found.clear(),
                _ => {}
            }
            found.push((rank, im.trait_.clone(), member));
        }
        found.into_iter().map(|(_, r, m)| (r, m)).collect()
    }
    /// The member `name` impl `im` gives `recv`: its own fn, or its trait's default.
    fn impl_gives(
        &self,
        im: &ImplDecl,
        recv: &Ty,
        name: Sym,
        trait_: Option<TraitId>,
        pick: &impl Fn(&FnDecl) -> bool,
        sets: &[SetTy],
    ) -> Option<Member> {
        let tr = self.items.t(im.trait_.tr);
        if trait_.is_some_and(|t| t != im.trait_.tr) {
            return None;
        }
        if let Some(m) = im
            .methods
            .get(&name)
            .copied()
            .filter(|&m| pick(self.items.f(m)))
        {
            let targs = self.fits_impl(im, recv)?;
            return Some(Member::Method(m, targs, sets.to_vec()));
        }
        let tm = tr.methods.get(&name).copied()?;
        if self.items.f(tm).bodiless() || !pick(self.items.f(tm)) {
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
    }
    /// What a derived impl of `r` for `s` asks of `s`'s parameters: each field's type, or each
    /// variant's, takes its impl of `r`, whose bounds go down to the parameters they name
    /// (`Range<R>: Read` asks `R: Ordered`). Its `@cache` fields count only when it `caches`.
    /// `None` when a field has no impl (R08.1.6).
    pub(super) fn derived_bounds(
        &self,
        s: StructId,
        r: &TraitRef,
        caches: bool,
    ) -> Option<Vec<(usize, TraitRef)>> {
        let sd = self.items.s(s);
        let mut out = Vec::new();
        // A derived impl meets its supertraits' derived bounds too: `<` compares the leading
        // fields with `==` (R08.1.1).
        let arity = self.items.t(r.tr).defaults.len();
        let supers = (self.items.t(r.tr).supers.iter())
            .filter(|sup| self.items.t(sup.tr).defaults.len() == arity)
            .map(|sup| TraitRef {
                tr: sup.tr,
                args: r.args.clone(),
            });
        let asks: Vec<TraitRef> = std::iter::once(r.clone()).chain(supers).collect();
        for ((_, t), cached) in sd.fields.iter().zip(&sd.cached) {
            if (caches || !*cached) && !asks.iter().all(|a| self.needs(t, a, &mut out, 32)) {
                return None;
            }
        }
        for (_, v) in &sd.variants {
            for b in self.derived_bounds(*v, r, caches)? {
                if !out.contains(&b) {
                    out.push(b);
                }
            }
        }
        Some(out)
    }
    /// Whether `t` can have `r`, pushing to `out` what that asks of the parameters `t` names:
    /// the first impl in `impls_of` order whose bounds can all be met. The cap only guards a
    /// blanket impl bounded by its own trait.
    fn needs(&self, t: &Ty, r: &TraitRef, out: &mut Vec<(usize, TraitRef)>, cap: usize) -> bool {
        match t {
            Ty::Param(i) => {
                let b = (*i, r.clone());
                if !out.contains(&b) {
                    out.push(b);
                }
                return true;
            }
            Ty::Var(_) | Ty::Error => return true,
            _ if cap == 0 => return false,
            _ => {}
        }
        (self.impls_of(r.tr)).any(|im| {
            let Some(targs) = self.impl_args_for(im, t, &r.args) else {
                return false;
            };
            let sub = Subst {
                tys: &targs,
                sets: &[],
            };
            let mut asked = out.clone();
            let met = im.bounds.iter().all(|(i, b)| {
                let b = TraitRef {
                    tr: b.tr,
                    args: b.args.iter().map(|x| sub.ty(x)).collect(),
                };
                (targs.get(*i)).is_none_or(|a| self.needs(a, &b, &mut asked, cap - 1))
            });
            if met {
                *out = asked;
            }
            met
        })
    }
    /// The impls of `tr`, in the order a type takes them (`impl_rank`).
    pub fn impls_of(&self, tr: TraitId) -> impl Iterator<Item = &'a ImplDecl> {
        self.ranked_impls(tr).map(|(_, im)| im)
    }
    /// `impls_of`, each with its index.
    pub fn ranked_impls(&self, tr: TraitId) -> impl Iterator<Item = (ImplIdx, &'a ImplDecl)> {
        let items = self.items;
        let rows = (self.rows.iter().enumerate())
            .filter(move |(_, im)| im.trait_.tr == tr)
            .map(|(j, im)| (ImplIdx::Row(j), im));
        let ranked = items.ranked(tr);
        let unranked = ranked.is_none().then(|| {
            (0..3).flat_map(move |r| {
                (Impls::of(items).trait_impls(tr)).filter(move |(_, im)| impl_rank(items, im) == r)
            })
        });
        rows.chain(
            ranked
                .into_iter()
                .flatten()
                .map(move |&k| (ImplIdx::Decl(k), items.imp(k))),
        )
        .chain(unranked.into_iter().flatten())
    }
    /// The impl's type arguments for `t`; a blanket impl only where `t` meets its bounds.
    pub fn fits_impl(&self, im: &ImplDecl, t: &Ty) -> Option<Vec<Ty>> {
        let targs = self.impl_args(im, t)?;
        (!im.blanket() || self.bounds_met(im, &targs)).then_some(targs)
    }
    fn bounds_met(&self, im: &ImplDecl, targs: &[Ty]) -> bool {
        (im.bounds.iter()).all(|(i, b)| targs.get(*i).is_none_or(|a| self.has_impl(a, b.tr)))
    }
    /// The bound at the root of `t` not meeting `r`: `Tuple1<List<?Int>>` misses `ShowAll`
    /// because `?Int` misses `Show`. It goes down while one impl alone of the most specific
    /// rank has the head `t` and its trait arguments, through the first of that impl's bounds
    /// left unmet.
    pub fn unmet_root(&self, t: &Ty, r: &TraitRef) -> (Ty, TraitRef) {
        let (mut t, mut r) = (t.clone(), r.clone());
        // An impl's bound names a part of its type, so each step is a smaller type; the cap
        // only guards a blanket impl bounded by its own trait.
        for _ in 0..32 {
            let heads: Vec<(&ImplDecl, Vec<Ty>)> = (self.impls_of(r.tr))
                .filter_map(|im| Some((im, self.impl_args_for(im, &t, &r.args)?)))
                .collect();
            let Some(best) = heads.iter().map(|(im, _)| impl_rank(self.items, im)).min() else {
                break;
            };
            let mut heads = heads
                .into_iter()
                .filter(|(im, _)| impl_rank(self.items, im) == best);
            let (Some((im, targs)), None) = (heads.next(), heads.next()) else {
                break;
            };
            let sub = Subst {
                tys: &targs,
                sets: &[],
            };
            let unmet = im.bounds.iter().find_map(|(i, b)| {
                let a = targs.get(*i)?;
                let b = TraitRef {
                    tr: b.tr,
                    args: b.args.iter().map(|x| sub.ty(x)).collect(),
                };
                (!self.meets(a, &b)).then(|| (a.clone(), b))
            });
            // A blanket impl stands for the trait only when it is the trait's one impl at these
            // arguments (`impl<T: Show, M> ShowPart<M> for T`); one among many (a `Show` for
            // every `Domain`) is not why `?Int` lacks `Show`.
            let sole = || {
                (self.impls_of(r.tr))
                    .filter(|o| self.impl_args_for(o, &Ty::Error, &r.args).is_some())
                    .count()
                    == 1
            };
            let unmet = unmet.filter(|_| !im.blanket() || sole());
            let Some(next) = unmet else {
                break;
            };
            (t, r) = next;
        }
        (t, r)
    }
    /// Whether `t` has an impl of `r` with its trait arguments, its bounds met as `has_impl`
    /// meets them.
    fn meets(&self, t: &Ty, r: &TraitRef) -> bool {
        if matches!(t, Ty::Param(_) | Ty::Var(_) | Ty::Error) {
            return true;
        }
        (self.impls_of(r.tr)).any(|im| {
            self.impl_args_for(im, t, &r.args)
                .is_some_and(|targs| self.bounds_met(im, &targs))
        })
    }
    /// Whether `t` has an impl of `tr`, its parameters taken to meet the impl's bounds.
    pub fn has_impl(&self, t: &Ty, tr: TraitId) -> bool {
        if matches!(t, Ty::Param(_) | Ty::Var(_) | Ty::Error) {
            return true;
        }
        (self.impls_of(tr)).any(|im| {
            self.impl_args(im, t)
                .is_some_and(|targs| self.bounds_met(im, &targs))
        })
    }
    /// An impl for `_..=_` is for every range type and never `Int`; one for `Int` is for a range
    /// type only when its trait has none for `_..=_`.
    pub fn head_fits(&self, im: &ImplDecl, t: &Ty) -> bool {
        match t {
            t if t.is_int_range() && im.for_ == Ty::Int => {
                !(self.impls_of(im.trait_.tr)).any(|o| matches!(o.for_, Ty::AnyRange(_)))
            }
            _ => true,
        }
    }
    /// The impl's type arguments when it is for `t`; a type not known yet fits any.
    pub fn impl_args(&self, im: &ImplDecl, t: &Ty) -> Option<Vec<Ty>> {
        self.impl_args_for(im, t, &[])
    }
    /// The impl's type arguments when it is for `t` and of the trait with arguments `trait_args`
    /// (`impl Into<Char> for Int` and not `impl Into<U64> for Int`); fewer are not checked.
    pub fn impl_args_for(&self, im: &ImplDecl, t: &Ty, trait_args: &[Ty]) -> Option<Vec<Ty>> {
        // A struct's impl fits only that struct or a type not known yet, as `fit` would say.
        if let Ty::Struct(d, ..) = im.for_ {
            if !matches!(t, Ty::Struct(e, ..) if *e == d) && !matches!(t, Ty::Var(_) | Ty::Error) {
                return None;
            }
        }
        if !self.head_fits(im, t) {
            return None;
        }
        let mut binds = vec![None; im.generics.len()];
        if !fit(&im.for_, t, &mut binds)
            || !(self.items.t(im.trait_.tr).inputs(&im.trait_.args).iter())
                .zip(self.items.t(im.trait_.tr).inputs(trait_args))
                .all(|(p, a)| fit(p, a, &mut binds))
        {
            return None;
        }
        // A parameter only a bound names (`D` of `Mapped<S: Indexed<D, T>, T, U>`) is what
        // the impl meeting that bound fixes it to (R08.1.2).
        loop {
            let unbound = binds.iter().filter(|b| b.is_none()).count();
            for (i, r) in &im.bounds {
                // A type not known yet fits every impl, so none fixes the parameter yet.
                let Some(bt) = binds[*i]
                    .clone()
                    .filter(|t| !matches!(t, Ty::Var(_) | Ty::Error))
                else {
                    continue;
                };
                // Only the one impl of the most specific rank that meets it fixes them (R08.1.4).
                let mut fits = (self.impls_of(r.tr)).filter_map(|m| {
                    let b = self.fits_impl(m, &bt)?;
                    let sub = Subst { tys: &b, sets: &[] };
                    let args: Vec<Ty> = m.trait_.args.iter().map(|a| sub.ty(a)).collect();
                    Some((impl_rank(self.items, m), args))
                });
                let Some((rank, args)) = fits.next() else {
                    continue;
                };
                if fits.next().is_some_and(|(k, _)| k == rank) {
                    continue;
                }
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
pub(super) fn supertraits_unmet(
    d: &dyn Signatures,
    im: &ImplDecl,
) -> Vec<(SourceId, cpl_diag::Code, String)> {
    let mut out = Vec::new();
    {
        let tr = d.t(im.trait_.tr);
        for sup in &tr.supers {
            let met = match &im.for_ {
                Ty::Param(i) => im.bounds.iter().any(|(j, r)| j == i && r.tr == sup.tr),
                // Every range type meets the same impls.
                Ty::AnyRange(_) => (d
                    .impls()
                    .ranked_impls(sup.tr)
                    .map(|(k, _)| k)
                    .collect::<Vec<_>>()
                    .into_iter())
                .any(|o| {
                    d.impls()
                        .impl_args(d.impls().imp(o), &Ty::Range(0, 0))
                        .is_some()
                }),
                t => (d
                    .impls()
                    .ranked_impls(sup.tr)
                    .map(|(k, _)| k)
                    .collect::<Vec<_>>()
                    .into_iter())
                .any(|o| d.impls().impl_args(d.impls().imp(o), t).is_some()),
            };
            if !met {
                out.push((
                    im.at.clone(),
                    cpl_diag::codes::MISSING_IMPL,
                    format!(
                        "`impl {}` needs an `impl {}` for the same type (R08.1.1)",
                        tr.name,
                        d.t(sup.tr).name
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
                // A parameter has it by its bound, or by a blanket impl its bounds meet.
                Ty::Param(i) => {
                    let has = |tr: TraitId| im.bounds.iter().any(|(j, r)| j == i && r.tr == tr);
                    has(ab.tr)
                        || (d
                            .impls()
                            .ranked_impls(ab.tr)
                            .map(|(k, _)| k)
                            .collect::<Vec<_>>()
                            .into_iter())
                        .any(|o| match d.impls().imp(o).for_ {
                            Ty::Param(k) => (d.impls().imp(o).bounds.iter())
                                .filter(|(j, _)| *j == k)
                                .all(|(_, r)| has(r.tr)),
                            _ => false,
                        })
                }
                Ty::Error => true,
                t => d.impls().has_impl(t, ab.tr),
            };
            if !met {
                let k = a - (tr.defaults.len() + 1 - tr.assoc.len());
                out.push((
                    im.assoc_at.get(k).cloned().unwrap_or(im.at.clone()),
                    cpl_diag::codes::MISSING_IMPL,
                    format!(
                        "`impl {}` chooses a type for `{}` with no `impl {}` (R08.1.1)",
                        tr.name,
                        tr.assoc[k],
                        d.t(ab.tr).name
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
        (Ty::Raw(x), Ty::Raw(y)) | (Ty::Ref(x), Ty::Ref(y)) | (Ty::Mut(x), Ty::Mut(y)) => {
            same_shape(x, y)
        }
        (Ty::Struct(d1, x, _), Ty::Struct(d2, y, _)) => {
            d1 == d2 && x.iter().zip(y).all(|(x, y)| same_shape(x, y))
        }
        (Ty::Clo(x), Ty::Clo(y)) => same_item(x, y),
        _ => a == b,
    }
}

// Two types that may yet be one: `Optional<Int>` and `Optional<_>`.
fn agree(a: &Ty, b: &Ty) -> bool {
    match (a, b) {
        (Ty::Var(_) | Ty::Error, _) | (_, Ty::Var(_) | Ty::Error) => true,
        (Ty::Raw(a), Ty::Raw(b)) | (Ty::Ref(a), Ty::Ref(b)) | (Ty::Mut(a), Ty::Mut(b)) => {
            agree(a, b)
        }
        (Ty::Struct(d1, a, _), Ty::Struct(d2, b, _)) => {
            d1 == d2 && a.len() == b.len() && a.iter().zip(b).all(|(x, y)| agree(x, y))
        }
        (Ty::Clo(x), Ty::Clo(y)) => same_item(x, y),
        _ => a == b,
    }
}
fn fit(p: &Ty, t: &Ty, binds: &mut [Option<Ty>]) -> bool {
    match (p, t) {
        (Ty::AnyRange(i), t) if t.is_int_range() || matches!(t, Ty::Var(_) | Ty::Error) => {
            fit(&Ty::Param(*i), t, binds)
        }
        (Ty::AnyRange(_), _) => false,
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
        (Ty::Raw(a), Ty::Raw(b)) | (Ty::Ref(a), Ty::Ref(b)) | (Ty::Mut(a), Ty::Mut(b)) => {
            fit(a, b, binds)
        }
        (Ty::RangeTo(a, m, x, k), Ty::RangeTo(b, n, y, j)) => {
            m == n && x == y && k == j && fit(a, b, binds)
        }
        (Ty::Struct(d1, a, _), Ty::Struct(d2, b, _)) => {
            d1 == d2 && a.len() == b.len() && a.iter().zip(b).all(|(x, y)| fit(x, y, binds))
        }
        (r, Ty::Int) | (Ty::Int, r) if r.is_int_range() => true,
        (Ty::Clo(x), Ty::Clo(y)) => same_item(x, y),
        // A fn type is a type like any other: `impl<T> Op<T> for (T, T) -> T` fits any set.
        (Ty::Fn(pa, ra, _), Ty::Fn(pb, rb, _)) => {
            pa.len() == pb.len()
                && pa.iter().zip(pb).all(|(x, y)| fit(x, y, binds))
                && fit(ra, rb, binds)
        }
        _ => p == t,
    }
}
/// One fn item, whatever its instance: a generic fn has one `type_of` (R08.1.10).
fn same_item(a: &CloRef, b: &CloRef) -> bool {
    match (a, b) {
        (CloRef::Item(x), CloRef::Item(y)) => x.decl == y.decl,
        _ => a == b,
    }
}

/// The closure sets an impl's fn-type head takes from `recv`: each `SetTy::Param(k)` it holds
/// is the set at that place in `recv` (`impl Named for (Int, Int) -> Int` on a lambda's type).
fn head_sets(head: &Ty, recv: &Ty, out: &mut Vec<SetTy>) {
    let (Ty::Fn(hp, hr, hs), Ty::Fn(rp, rr, rs)) = (head, recv) else {
        return;
    };
    if let SetTy::Param(k) = hs {
        if out.len() <= *k {
            out.resize(*k + 1, SetTy::Closed(Vec::new()));
        }
        out[*k] = rs.clone();
    }
    for (h, r) in hp.iter().zip(rp) {
        head_sets(h, r, out);
    }
    head_sets(hr, rr, out);
}

/// The library method `name` whose `self` can be `recv`, from any std module whether the
/// program loads it or not: a method call needs no import (R11.3.4).
pub fn method_for(items: &dyn Items, m: &dyn Methods, name: Sym, recv: &Ty) -> Option<FnId> {
    (m.methods_named(name).iter().copied()).find(|f| self_fits(items, *f, recv))
}

/// The fn of an impl over a shape of `recv`'s struct (`impl Rev<[D]T>:`) named `name` whose
/// `self` `recv` fits, as a trait impl's header is matched (R08.1.3).
pub fn shaped_member(items: &dyn Items, m: &dyn Methods, recv: &Ty, name: Sym) -> Option<FnId> {
    let Ty::Struct(s, ..) = recv else {
        return None;
    };
    (m.shaped(*s, name).iter().copied()).find(|f| {
        let f = items.f(*f);
        let mut binds = vec![None; f.generics.len()];
        f.self_ty
            .as_ref()
            .is_some_and(|st| fit(st, recv, &mut binds))
    })
}

/// Whether `recv` has the shape of `f`'s `self`: a struct whose arguments fit it, or the same kind of type.
pub fn self_fits(items: &dyn Items, f: FnId, recv: &Ty) -> bool {
    let f = items.f(f);
    match (f.self_ty.as_ref(), recv) {
        (Some(Ty::Param(_)), _) | (_, Ty::Var(_) | Ty::Error) => true,
        (Some(st @ Ty::Struct(..)), _) => fit(st, recv, &mut vec![None; f.generics.len()]),
        // `self: 0..` takes an `Int` receiver as a parameter of that type takes an argument.
        (Some(st), Ty::Int) if st.is_int_range() => true,
        // A fn-typed `self` takes a closure's own type, coerced in as an argument is.
        (Some(Ty::Fn(..)), Ty::Clo(_)) => true,
        (Some(t), r) => std::mem::discriminant(t) == std::mem::discriminant(r),
        (None, _) => false,
    }
}
