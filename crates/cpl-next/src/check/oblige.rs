//! Trait bounds a call must meet (R08.1.2): each is met by the caller's own bounds or by one
//! impl, and meeting it fixes the trait's arguments, so `sorted<S: Seq<T>, T>` learns `T`.
use super::*;
use crate::decls::TraitRef;

impl Cx<'_> {
    /// The bounds `f`'s type arguments must meet at this call.
    pub(super) fn oblige(&mut self, f: &FnRef, span: Span) {
        let fd = self.d.f(f.decl);
        let sub = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        for (i, r) in &fd.bounds {
            let r = TraitRef {
                tr: r.tr,
                args: r.args.iter().map(|a| sub.ty(a)).collect(),
            };
            self.obligations
                .push((f.targs[*i].clone(), r, span, Origin::Bound));
        }
    }
    /// Meets each bound whose type is known by now; `last` drops those never known.
    pub(super) fn solve(&mut self, last: bool) {
        loop {
            let pending = std::mem::take(&mut self.obligations);
            // A bound met by an impl may leave the impl's own bounds: they count as progress too.
            let (before, mut waiting) = (pending.len(), 0);
            for (t, r, span, why) in pending {
                if self.unknown(&t) {
                    self.obligations.push((t, r, span, why));
                    waiting += 1;
                    continue;
                }
                let t = self.zonk(&t);
                match self.meet(&t, &r, span) {
                    Some(true) => {}
                    Some(false) if self.mentions_error(&t) => self.poison(&r),
                    Some(false) => {
                        self.poison(&r);
                        let tr = self.d.traits[r.tr.0].name.clone();
                        let shown = self.show(&t);
                        let record = matches!(&t, Ty::Struct(d, ..) if crate::records::is_record(&self.d.s(*d).name));
                        if record {
                            let msg = format!("slice: `{shown}` meeting `{tr}` by its fields");
                            self.err(span, msg);
                            continue;
                        }
                        if why == Origin::Convert {
                            let to = (r.args.iter().map(|a| self.show(a)))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let msg = format!(
                                "{}: expected `{to}`, found `{shown}`",
                                cpl_diag::codes::TYPE_MISMATCH.0
                            );
                            let help = format!(
                                "`{shown}` converts to `{to}` only by its own impl: write `impl {tr}<{to}> for {shown}`"
                            );
                            self.err_help(span, msg, Some(help));
                            continue;
                        }
                        self.err(
                            span,
                            format!(
                                "{}: `{shown}` does not implement `{tr}`",
                                cpl_diag::codes::MISSING_IMPL.0
                            ),
                        );
                    }
                    None => {
                        self.obligations.push((t, r, span, why));
                        waiting += 1;
                    }
                }
            }
            if waiting == before {
                break;
            }
        }
        if last {
            self.obligations.clear();
        }
    }
    /// The fn type a lambda of `arity` parameters given for `v` is: the one impl of a trait
    /// bounding `v` that targets a fn type of that arity (`impl<T> Op<T> for (T, T) -> T`), its
    /// parameters fixed by the bound's arguments. `v` becomes that fn type, so the lambda passes
    /// as a value of it.
    pub(super) fn fn_impl_for(&mut self, v: usize, arity: usize) -> Option<Ty> {
        let d = self.d;
        let bounds: Vec<TraitRef> = (self.obligations.iter())
            .filter(|(t, ..)| self.shallow(t) == Ty::Var(v))
            .map(|(_, r, ..)| r.clone())
            .collect();
        let found: Vec<(usize, &TraitRef)> = (bounds.iter())
            .flat_map(|r| {
                (d.trait_impls(r.tr))
                    .filter(|(_, im)| matches!(&im.for_, Ty::Fn(ps, ..) if ps.len() == arity))
                    .map(move |(k, _)| (k, r))
            })
            .collect();
        let [(k, r)] = found[..] else {
            return None;
        };
        let im = &d.impls[k];
        let Ty::Fn(ps, ret, _) = &im.for_ else {
            return None;
        };
        let targs: Vec<Ty> = im.generics.iter().map(|_| self.new_ty()).collect();
        let sub = Subst {
            tys: &targs,
            sets: &[],
        };
        let f = Ty::Fn(
            ps.iter().map(|p| sub.ty(p)).collect(),
            Box::new(sub.ty(ret)),
            self.new_set(),
        );
        let args: Vec<Ty> = im.trait_.args.iter().map(|a| sub.ty(a)).collect();
        let fits = args.iter().zip(&r.args).all(|(a, b)| self.unify(a, b));
        (fits && self.unify(&Ty::Var(v), &f)).then_some(f)
    }
    /// Whether a pending bound is on the inference variable `v`: it, not a default, fixes `v`.
    pub(super) fn obliged(&self, v: usize) -> bool {
        (self.obligations.iter()).any(|(t, ..)| self.shallow(t) == Ty::Var(v))
    }
    /// Whether some part of `t` is still an inference variable.
    fn unknown(&self, t: &Ty) -> bool {
        self.holds(t, &|t| matches!(t, Ty::Var(_)))
    }
    /// Whether `t` is built from an error node's type: a verdict on it is downstream of that error.
    pub(super) fn mentions_error(&self, t: &Ty) -> bool {
        self.holds(t, &|t| *t == Ty::Error)
    }
    /// A bound that failed solves none of its arguments: what they leave unknown is the error
    /// node's type, so nothing reached through them is diagnosed again.
    fn poison(&mut self, r: &TraitRef) {
        let open = std::cell::RefCell::new(Vec::new());
        for a in &r.args {
            self.holds(a, &|t| {
                matches!(t, Ty::Var(x) if { open.borrow_mut().push(*x); true }) && false
            });
        }
        for x in open.into_inner() {
            self.tys[x] = Some(Ty::Error);
        }
    }
    fn holds(&self, t: &Ty, leaf: &dyn Fn(&Ty) -> bool) -> bool {
        match self.shallow(t) {
            t if leaf(&t) => true,
            Ty::Raw(e) | Ty::Ref(e) => self.holds(&e, leaf),
            Ty::Struct(_, ts, _) => ts.iter().any(|t| self.holds(t, leaf)),
            Ty::Fn(ps, r, _) => ps.iter().any(|t| self.holds(t, leaf)) || self.holds(&r, leaf),
            _ => false,
        }
    }
    /// Whether `t` meets `r`, or `None` while `t` is not known.
    fn meet(&mut self, t: &Ty, r: &TraitRef, span: Span) -> Option<bool> {
        let tr = &self.d.traits[r.tr.0];
        if tr.alias {
            let sub = Subst {
                tys: &r.args,
                sets: &[],
            };
            let needs: Vec<TraitRef> = (tr.supers.iter())
                .map(|s| TraitRef {
                    tr: s.tr,
                    args: s.args.iter().map(|a| sub.ty(a)).collect(),
                })
                .collect();
            let t = t.clone();
            self.obligations.extend(
                needs
                    .into_iter()
                    .map(|n| (t.clone(), n, span, Origin::Bound)),
            );
            return Some(true);
        }
        // A bound the fn states is met as stated; otherwise a parameter meets a trait through a
        // blanket impl whose bounds it meets (`impl<D: Domain> Eq for D`), like any other type.
        if let Ty::Param(i) = t {
            let tr = &self.d.traits[r.tr.0];
            let mut have = self.bound_traits(*i).into_iter().filter(|h| h.tr == r.tr);
            let fitting: Vec<TraitRef> = (have.clone())
                .filter(|h| {
                    (tr.inputs(&h.args).iter().zip(tr.inputs(&r.args)))
                        .all(|(a, b)| self.fits(a, b))
                })
                .collect();
            let Some(h) = fitting.first().cloned().or_else(|| have.next()) else {
                return Some(self.blanket(*i, r.tr));
            };
            if fitting.len() > 1 && tr.inputs(&r.args).iter().any(|a| self.unknown(a)) {
                return None;
            }
            return Some(h.args.iter().zip(&r.args).all(|(a, b)| self.unify(a, b)));
        }
        // Every domain meets `Domain`; any other type by an impl.
        if Some(r.tr) == self.d.domain_trait && matches!(t, Ty::Dom(_)) {
            return Some(true);
        }
        match t {
            Ty::Error => Some(true),
            _ => {
                // The impl whose trait arguments fit too: `Int` has `Into<Char>` and `Into<U64>`.
                let want: Vec<Ty> = (self.d.traits[r.tr.0].inputs(&r.args).iter())
                    .map(|a| self.zonk(a))
                    .collect();
                let mut found: Vec<(usize, Vec<Ty>)> = (self.d.trait_impls(r.tr))
                    .filter_map(|(k, im)| Some((k, self.d.impl_args_for(im, t, &want)?)))
                    .filter(|(k, _)| {
                        !self.d.impls[*k].blanket()
                            || self.d.fits_impl(&self.d.impls[*k], t).is_some()
                    })
                    .collect();
                // An impl for a written type beats a blanket one (R08.1.2).
                if found.iter().any(|(k, _)| !self.d.impls[*k].blanket()) {
                    found.retain(|(k, _)| !self.d.impls[*k].blanket());
                }
                if found.len() > 1 {
                    if self.unknown(t) || want.iter().any(|a| self.unknown(a)) {
                        return None;
                    }
                    let (tr, shown) = (self.d.traits[r.tr.0].name.clone(), self.show(t));
                    let msg = format!(
                        "{}: `{shown}` has {} impls of `{tr}` that fit here: write the trait's arguments",
                        cpl_diag::codes::AMBIGUOUS_METHOD.0,
                        found.len()
                    );
                    self.err(span, msg);
                    return Some(true);
                }
                let Some((k, targs)) = found.into_iter().next() else {
                    return Some(false);
                };
                // A parameter the type does not fix (`impl Keyed<Int, T> for Str`) is fixed by the bound.
                let targs: Vec<Ty> = (targs.into_iter())
                    .map(|a| if a == Ty::Error { self.new_ty() } else { a })
                    .collect();
                let im = &self.d.impls[k];
                let sub = Subst {
                    tys: &targs,
                    sets: &[],
                };
                let args: Vec<Ty> = im.trait_.args.iter().map(|a| sub.ty(a)).collect();
                let needs: Vec<(Ty, TraitRef)> = (im.bounds.iter())
                    .map(|(i, b)| {
                        let b = TraitRef {
                            tr: b.tr,
                            args: b.args.iter().map(|a| sub.ty(a)).collect(),
                        };
                        (targs[*i].clone(), b)
                    })
                    .collect();
                self.obligations
                    .extend(needs.into_iter().map(|(t, b)| (t, b, span, Origin::Bound)));
                Some(args.iter().zip(&r.args).all(|(a, b)| self.unify(a, b)))
            }
        }
    }
    /// Whether type parameter `i` meets `tr` through a blanket impl whose bounds its own bounds
    /// meet: `impl<D: Domain> Eq for D` gives every `D: Domain` its `==`.
    /// A domain parameter (`Decls::domain_params`) meets `Domain` and what `Domain` gives.
    pub(super) fn blanket(&self, i: usize, tr: TraitId) -> bool {
        let mut have: Vec<TraitId> = self.bound_traits(i).iter().map(|h| h.tr).collect();
        let dom = (self.decl).and_then(|f| self.d.domain_params.get(&f));
        if dom.is_some_and(|ds| ds.contains(&i)) {
            have.extend(self.d.domain_trait);
        }
        if have.contains(&tr) {
            return true;
        }
        (self.d.trait_impls(tr)).any(|(_, im)| {
            im.trait_.args.is_empty()
                && matches!(im.for_, Ty::Param(_))
                && (im.bounds.iter())
                    .filter(|(k, _)| im.for_ == Ty::Param(*k))
                    .all(|(_, b)| have.contains(&b.tr))
        })
    }
    /// The traits the fn being checked bounds its type parameter `i` by, with their supertraits (R08.1.1).
    pub(super) fn bound_traits(&self, i: usize) -> Vec<TraitRef> {
        let Some(decl) = self.decl else {
            return Vec::new();
        };
        let mut traits: Vec<TraitRef> = (self.d.f(decl).bounds.iter())
            .filter(|(pi, _)| *pi == i)
            .map(|(_, r)| r.clone())
            .collect();
        let dom = self
            .d
            .domain_params
            .get(&decl)
            .is_some_and(|ds| ds.contains(&i));
        if let Some(tr) = self.d.domain_trait.filter(|_| dom) {
            let r = TraitRef {
                tr,
                args: Vec::new(),
            };
            if !traits.contains(&r) {
                traits.push(r);
            }
        }
        let mut k = 0;
        while k < traits.len() {
            let r = traits[k].clone();
            for sup in &self.d.traits[r.tr.0].supers {
                let sub = Subst {
                    tys: &r.args,
                    sets: &[],
                };
                let s = TraitRef {
                    tr: sup.tr,
                    args: sup.args.iter().map(|t| sub.ty(t)).collect(),
                };
                if !traits.contains(&s) {
                    traits.push(s);
                }
            }
            k += 1;
        }
        traits
    }
}
