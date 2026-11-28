//! Trait bounds a call must meet (R08.1.2): each is met by the caller's own bounds or by one
//! impl, and meeting it fixes the trait's arguments, so `sorted<S: Seq<T>, T>` learns `T`.
use super::*;
use cpl_next_decls::decls::ImplIdx;
use cpl_next_ir::unbuilt::Unbuilt;
use decls::TraitRef;

impl Cx<'_> {
    /// The bounds `f`'s type arguments must meet at this call.
    pub(super) fn oblige(&mut self, f: &FnRef, span: Span) {
        let fd = self.items.f(f.decl);
        let sub = Subst {
            tys: &f.targs,
            sets: &f.sets,
        };
        for (i, r) in &fd.bounds {
            let r = TraitRef {
                tr: r.tr,
                args: r.args.iter().map(|a| sub.ty(a)).collect(),
            };
            self.b
                .obligations
                .push((f.targs[*i].clone(), r, span, Origin::Bound));
        }
    }
    /// Meets each bound whose type is known by now; `last` drops those never known.
    pub(super) fn solve(&mut self, last: bool) {
        loop {
            let pending = std::mem::take(&mut self.b.obligations);
            // A bound met by an impl may leave the impl's own bounds: they count as progress too.
            let (before, mut waiting) = (pending.len(), 0);
            for (t, r, span, why) in pending {
                // A struct whose arguments are still open meets a trait by the one impl its head
                // takes, which can fix them (`Mon<?T>: Fold<Int>` by `impl<T> Fold<T> for Mon<T>`).
                let partial = self.unknown(&t);
                if partial && !matches!(self.shallow(&t), Ty::Struct(..)) {
                    self.b.obligations.push((t, r, span, why));
                    waiting += 1;
                    continue;
                }
                let open = t.clone();
                let t = self.zonk(&t);
                if partial {
                    let Some((k, _)) = self.head_impl(&t, &r) else {
                        self.b.obligations.push((t, r, span, why));
                        continue;
                    };
                    // Its arguments are `open`'s own, not the zonked `t`'s: an open part of it
                    // stays open, with its sets.
                    let targs = vec![Ty::Error; self.impls().imp(k).generics.len()];
                    if self.take_impl(k, targs, &r, span, Some(&open)) {
                        continue;
                    }
                }
                match self.meet(&t, &open, &r, span) {
                    Some(true) => {}
                    Some(false) if self.mentions_error(&t) => self.poison(&r),
                    Some(false) => {
                        self.poison(&r);
                        let tr = self.items.t(r.tr).name.clone();
                        let shown = self.show(&t);
                        // A type no impl converts to has no `T(x)` at all; one some impl converts to
                        // takes only what converts (`List(5)` is a mismatch, `Deque(xs)` is not a call).
                        let converts = why == Origin::Construct
                            && (self.impls().impls_of(r.tr)).any(|im| {
                                matches!((&im.trait_.args[..], self.shallow(&r.args[0])),
                                    ([Ty::Struct(a, ..)], Ty::Struct(b, ..)) if *a == b)
                            });
                        if why == Origin::Construct && !converts {
                            let to = match self.shallow(&r.args[0]) {
                                Ty::Struct(d, ..) => self.items.s(d).name.clone(),
                                t => self.show(&t),
                            };
                            let msg = Msg::verdict(
                                cpl_diag::codes::WRONG_ARG_COUNT,
                                "expected 0 arguments, found 1".to_string(),
                            );
                            let help = format!(
                                "`{to}`'s `new` takes nothing, and `{shown}` has no `impl {tr}<{to}>` to convert it"
                            );
                            self.err_help(span, msg, Some(help));
                            continue;
                        }
                        if matches!(why, Origin::Convert | Origin::Construct) {
                            let to = (r.args.iter().map(|a| self.show(a)))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let msg = Msg::verdict(
                                cpl_diag::codes::TYPE_MISMATCH,
                                format!("expected `{to}`, found `{shown}`"),
                            );
                            let help = format!(
                                "`{shown}` converts to `{to}` only by its own impl: write `impl {tr}<{to}> for {shown}`"
                            );
                            self.err_help(span, msg, Some(help));
                            continue;
                        }
                        // Named at its root: `print(xs)` on a `List<?Int>` is `?Int` lacking
                        // `Show`, not the pack of arguments lacking `ShowAll` (R10.3.7).
                        let (root, rr) = self.impls().unmet_root(&t, &r);
                        let (shown, tr) = (self.show(&root), &self.items.t(rr.tr).name);
                        let said = &self.items.t(rr.tr).unimplemented;
                        let code = said.code.unwrap_or(cpl_diag::codes::MISSING_IMPL);
                        let help = (said.help.as_ref()).map(|h| h.replace("{T}", &shown));
                        let msg =
                            Msg::verdict(code, format!("`{shown}` does not implement `{tr}`"));
                        self.err_help(span, msg, help);
                    }
                    None => {
                        self.b.obligations.push((t, r, span, why));
                        waiting += 1;
                    }
                }
            }
            if waiting == before {
                break;
            }
        }
        if last {
            // A bound still open at the end has one impl only if exactly one fits whatever it
            // leaves open: more is no choice (R08.1.2).
            for (t, r, span, _) in std::mem::take(&mut self.b.obligations) {
                let t = self.zonk(&t);
                let n = match t {
                    Ty::Param(i) => self.stated(i, &r).len(),
                    Ty::Var(_) | Ty::Error => continue,
                    _ if self.mentions_error(&t) => continue,
                    _ => self.fitting(&t, &r).len(),
                };
                if n > 1 {
                    self.ambiguous(&t, &r, n, span);
                }
            }
        }
    }
    /// The impls of `r` that fit `t`, of the most specific rank: one written for the type
    /// beats a blanket one (R08.1.2). `Int` has `Into<Char>` and `Into<U64>`: the trait's
    /// arguments, where known, choose among them.
    fn fitting(&self, t: &Ty, r: &TraitRef) -> Vec<(ImplIdx, Vec<Ty>)> {
        let r = TraitRef {
            tr: r.tr,
            args: r.args.iter().map(|a| self.zonk(a)).collect(),
        };
        self.impls().best(t, &r)
    }
    /// The bounds type parameter `i` states for `r`'s trait whose arguments fit `r`'s:
    /// `T: Cv<Int> + Cv<Bool>` meets `T: Cv<?U>` by both until `?U` is known.
    fn stated(&self, i: usize, r: &TraitRef) -> Vec<TraitRef> {
        let tr = self.items.t(r.tr);
        (self.bound_traits(i).into_iter())
            .filter(|h| h.tr == r.tr)
            .filter(|h| {
                (tr.inputs(&h.args).iter().zip(tr.inputs(&r.args))).all(|(a, b)| self.fits(a, b))
            })
            .collect()
    }
    fn ambiguous(&mut self, t: &Ty, r: &TraitRef, n: usize, span: Span) {
        let (tr, shown) = (self.items.t(r.tr).name.clone(), self.show(t));
        let msg = Msg::verdict(
            cpl_diag::codes::AMBIGUOUS_METHOD,
            format!("`{shown}` has {n} impls of `{tr}` that fit here: write the trait's arguments"),
        );
        self.err(span, msg);
    }
    /// The fn type a lambda of `arity` parameters given for `v` is: the one impl of a trait
    /// bounding `v` that targets a fn type of that arity (`impl<T> Op<T> for (T, T) -> T`), its
    /// parameters fixed by the bound's arguments. `v` becomes that fn type, so the lambda passes
    /// as a value of it.
    pub(super) fn fn_impl_for(&mut self, v: usize, arity: usize) -> Option<Ty> {
        let d = self.d;
        let bounds: Vec<TraitRef> = (self.b.obligations.iter())
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
        let im = d.imp(k);
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
    /// Whether `t` is an inference variable a pending bound may make a domain: it is bound by
    /// `Domain`, or is an argument an impl of a bound takes a domain for (`N: Next<V>`).
    pub(super) fn domain_var(&self, t: &Ty) -> bool {
        let t = self.shallow(t);
        matches!(t, Ty::Var(_))
            && (self.b.obligations.iter()).any(|(o, r, ..)| {
                (Some(r.tr) == self.d.roles().domain && self.shallow(o) == t)
                    || (r.args.iter().enumerate())
                        .any(|(j, a)| self.shallow(a) == t && self.facts.arg_may_be_domain(r.tr, j))
            })
    }
    /// Whether a pending bound is on the inference variable `v`: it, not a default, fixes `v`.
    pub(super) fn obliged(&self, v: usize) -> bool {
        (self.b.obligations.iter()).any(|(t, ..)| self.shallow(t) == Ty::Var(v))
    }
    /// Whether some part of `t` is still an inference variable.
    pub(super) fn unknown(&self, t: &Ty) -> bool {
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
            Ty::Raw(e) | Ty::Ref(e) | Ty::Mut(e) | Ty::RangeTo(e, ..) => self.holds(&e, leaf),
            Ty::Struct(_, ts, _) => ts.iter().any(|t| self.holds(t, leaf)),
            Ty::Fn(ps, r, _) => ps.iter().any(|t| self.holds(t, leaf)) || self.holds(&r, leaf),
            _ => false,
        }
    }
    /// Whether `t` meets `r`, or `None` while `t` is not known. `open` is `t` before zonking.
    fn meet(&mut self, t: &Ty, open: &Ty, r: &TraitRef, span: Span) -> Option<bool> {
        let tr = self.items.t(r.tr);
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
            self.b.obligations.extend(
                needs
                    .into_iter()
                    .map(|n| (t.clone(), n, span, Origin::Bound)),
            );
            return Some(true);
        }
        // A bound the fn states is met as stated; otherwise a parameter meets a trait through a
        // blanket impl whose bounds it meets (`impl<D: Domain> Eq for D`), like any other type.
        if let Ty::Param(i) = t {
            let tr = self.items.t(r.tr);
            let mut have = self.bound_traits(*i).into_iter().filter(|h| h.tr == r.tr);
            let mut fitting = self.stated(*i, r);
            fitting.dedup();
            if fitting.len() > 1 {
                // Two stated bounds fit: wait while the bound's own arguments are open, then
                // it is ambiguous, never the first one written.
                if tr.inputs(&r.args).iter().any(|a| self.unknown(a)) {
                    return None;
                }
                let (name, shown) = (tr.name.clone(), self.show(t));
                let text = format!(
                    "`{shown}` has {} bounds of `{name}` that fit here",
                    fitting.len()
                );
                let help = "write the trait's arguments, or keep one of the bounds".to_string();
                let msg = Msg::verdict(cpl_diag::codes::AMBIGUOUS_METHOD, text);
                self.err_help(span, msg, Some(help));
                return Some(true);
            }
            let Some(h) = fitting.pop().or_else(|| have.next()) else {
                let i = *i;
                return Some(match self.param_impl(i, r) {
                    Some(k) => {
                        let (n, for_) = {
                            let im = self.impls().imp(k);
                            (im.generics.len(), im.for_.clone())
                        };
                        let mut targs: Vec<Ty> = (0..n).map(|_| self.new_ty()).collect();
                        if let Ty::Param(p) = for_ {
                            targs[p] = Ty::Param(i);
                        }
                        self.take_impl(k, targs, r, span, None)
                    }
                    None => self.blanket(i, r.tr),
                });
            };
            return Some(h.args.iter().zip(&r.args).all(|(a, b)| self.unify(a, b)));
        }
        // A record meets a row trait by its fields, each field's type its argument (R08.4.1).
        if self.items.t(r.tr).row && !matches!(t, Ty::Error) {
            let Some(fs) = self.impls().row_fields(t, r) else {
                return Some(false);
            };
            return Some(
                (fs.iter()).all(|(i, ft)| r.args.get(*i).is_none_or(|a| self.unify(a, ft))),
            );
        }
        match t {
            Ty::Error => Some(true),
            _ => {
                // The impl whose trait arguments fit too: `Int` has `Into<Char>` and `Into<U64>`.
                let found = self.fitting(t, r);
                if found.len() > 1 {
                    // `want` is zonked, an open argument read as an error: the bound's own
                    // arguments say whether it is still open.
                    let inputs = self.items.t(r.tr).inputs(&r.args);
                    if self.unknown(t) || inputs.iter().any(|a| self.unknown(a)) {
                        return None;
                    }
                    self.ambiguous(t, r, found.len(), span);
                    return Some(true);
                }
                let Some((k, mut targs)) = found.into_iter().next() else {
                    return Some(false);
                };
                // The impl's arguments are `open`'s own parts where it has them, so a fn set
                // still filling (a call's result) stays the bound's set, not a copy of it now.
                let live = (self
                    .impls()
                    .impl_args(self.impls().imp(k), &self.followed(open)))
                .unwrap_or_default();
                for (a, l) in targs.iter_mut().zip(live) {
                    if l != Ty::Error {
                        *a = l;
                    }
                }
                Some(self.take_impl(k, targs, r, span, None))
            }
        }
    }
    /// `r` met by impl `k` at `targs`: its bounds become obligations and its trait arguments
    /// are the bound's, as its type is `open`'s when that is still open.
    fn take_impl(
        &mut self,
        k: ImplIdx,
        targs: Vec<Ty>,
        r: &TraitRef,
        span: Span,
        open: Option<&Ty>,
    ) -> bool {
        // A parameter the type does not fix (`impl Keyed<Int, T> for Str`) is fixed by the bound.
        let targs: Vec<Ty> = (targs.into_iter())
            .map(|a| if a == Ty::Error { self.new_ty() } else { a })
            .collect();
        let im = self.impls().imp(k);
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
        let for_ = sub.ty(&im.for_);
        self.b
            .obligations
            .extend(needs.into_iter().map(|(t, b)| (t, b, span, Origin::Bound)));
        let open = match (open.map(|t| self.shallow(t)), for_) {
            (Some(Ty::Struct(_, have, _)), Ty::Struct(_, want, _)) => {
                have.into_iter().zip(want).collect()
            }
            _ => Vec::new(),
        };
        open.iter().all(|(a, b)| self.unify(a, b))
            && args.iter().zip(&r.args).all(|(a, b)| self.unify(a, b))
    }
    /// The one blanket impl of `r` with trait arguments whose bounds on its target parameter
    /// `i`'s own bounds name: `S: Seq<T>` meets `Into<List<T>>` by
    /// `impl<S: Seq<T>, T> Into<List<T>> for S`; the bounds fix the impl's other parameters.
    fn param_impl(&self, i: usize, r: &TraitRef) -> Option<ImplIdx> {
        let have: Vec<TraitId> = self.bound_traits(i).iter().map(|h| h.tr).collect();
        let mut found = (self.impls().trait_impls(r.tr)).filter(|(_, im)| {
            !im.trait_.args.is_empty()
                && matches!(im.for_, Ty::Param(_))
                && (im.bounds.iter())
                    .filter(|(k, _)| im.for_ == Ty::Param(*k))
                    .all(|(_, b)| have.contains(&b.tr))
        });
        let (k, _) = found.next()?;
        found.next().is_none().then_some(k)
    }
    /// The one impl of `r` for a struct whose arguments are still open, by its head alone.
    fn head_impl(&self, t: &Ty, r: &TraitRef) -> Option<(ImplIdx, Vec<Ty>)> {
        let mut found = (self.d.impls.iter().enumerate())
            .filter(|(_, im)| im.trait_.tr == r.tr && !im.blanket())
            .filter_map(|(k, im)| Some((ImplIdx::Decl(k), self.impls().fits_impl(im, t)?)));
        let one = found.next()?;
        found.next().is_none().then_some(one)
    }
    /// Whether type parameter `i` meets `tr` through a blanket impl whose bounds its own bounds
    /// meet: `impl<D: Domain> Eq for D` gives every `D: Domain` its `==`.
    /// A domain parameter (`Decls::domain_params`) meets `Domain` and what `Domain` gives.
    pub(super) fn blanket(&self, i: usize, tr: TraitId) -> bool {
        let mut have: Vec<TraitId> = self.bound_traits(i).iter().map(|h| h.tr).collect();
        if (self.b.decl).is_some_and(|f| self.facts.domain_params(f).contains(&i)) {
            have.extend(self.d.roles().domain);
        }
        if (self.b.decl).is_some_and(|f| self.facts.ordered_params(f).contains(&i)) {
            have.extend(self.d.roles().order);
        }
        // Blankets chain: `impl<D: Domain + Eq> Ord for D` holds once `impl<D: Domain> Eq for D`
        // has given `Eq`.
        let blankets: Vec<&decls::ImplDecl> = (self.d.impls.iter())
            .filter(|im| im.trait_.args.is_empty() && matches!(im.for_, Ty::Param(_)))
            .collect();
        loop {
            if have.contains(&tr) {
                return true;
            }
            let given: Vec<TraitId> = (blankets.iter())
                .filter(|im| !have.contains(&im.trait_.tr))
                .filter(|im| {
                    (im.bounds.iter())
                        .filter(|(k, _)| im.for_ == Ty::Param(*k))
                        .all(|(_, b)| have.contains(&b.tr))
                })
                .map(|im| im.trait_.tr)
                .collect();
            if given.is_empty() {
                return false;
            }
            have.extend(given);
        }
    }
    /// The traits the fn being checked bounds its type parameter `i` by, with their supertraits (R08.1.1).
    /// `if T: Tr<A>:`: a test of `T` against the bound, which each instance decides (R08.2.9).
    pub(super) fn has_bound(&mut self, c: &next::BoundCond, span: Span) -> TExpr {
        let error = TExpr {
            kind: TK::Error,
            ty: Ty::Error,
            span,
        };
        let mut pre = Vec::new();
        let Some(subject) = c.subject() else {
            return error;
        };
        let t = self.body_ty(&subject, &mut pre);
        let mut bounds = c
            .bound()
            .into_iter()
            .flat_map(|b| b.types().collect::<Vec<_>>());
        let (Some(next::Type::Path(bound)), None) = (bounds.next(), bounds.next()) else {
            self.err(span, Unbuilt::BoundNotNamed);
            return error;
        };
        let name: Vec<String> = bound.segments().map(|s| s.text().to_string()).collect();
        let name = name.join(".");
        let found = self
            .syms
            .get(&name)
            .and_then(|n| self.names.named(self.b.module, n));
        let Some(decls::Named::Trait(tr)) = found else {
            let code = cpl_diag::codes::UNKNOWN_TYPE;
            let at = self.at(&bound);
            self.err(at, Msg::verdict(code, format!("unknown trait `{name}`")));
            return error;
        };
        let written: Vec<next::GenericArg> = bound
            .generic_args()
            .map_or_else(Vec::new, |g| g.args().collect());
        let mut args = Vec::new();
        for a in written {
            let next::GenericArg::Type(a) = a else {
                self.err(span, Unbuilt::BoundNotNamed);
                return error;
            };
            args.push(self.body_ty(&a, &mut pre));
        }
        let r = self.d.with_defaults(TraitRef { tr, args }, &t);
        if t == Ty::Error {
            return error;
        }
        TExpr {
            kind: TK::HasBound(t, r),
            ty: Ty::Bool,
            span,
        }
    }
    pub(super) fn bound_traits(&self, i: usize) -> Vec<TraitRef> {
        self.bounds_of(i, true)
    }
    /// `bound_traits`, or with `implied` false only those written: a stated bound or an
    /// `if T: Tr:`, not the `Domain` a domain parameter meets unwritten.
    pub(super) fn bounds_of(&self, i: usize, implied: bool) -> Vec<TraitRef> {
        let Some(decl) = self.b.decl else {
            return Vec::new();
        };
        let mut traits: Vec<TraitRef> = (self.items.f(decl).bounds.iter())
            .filter(|(pi, _)| *pi == i)
            .map(|(_, r)| r.clone())
            .collect();
        let dom = implied && self.facts.domain_params(decl).contains(&i);
        let ordered = implied && self.facts.ordered_params(decl).contains(&i);
        let implied = [
            self.d.roles().domain.filter(|_| dom),
            self.d.roles().order.filter(|_| ordered),
        ];
        for tr in implied.into_iter().flatten() {
            let r = TraitRef {
                tr,
                args: Vec::new(),
            };
            if !traits.contains(&r) {
                traits.push(r);
            }
        }
        for (j, r) in &self.b.assumed {
            if *j == i && !traits.contains(r) {
                traits.push(r.clone());
            }
        }
        let mut k = 0;
        while k < traits.len() {
            let r = traits[k].clone();
            for sup in &self.items.t(r.tr).supers {
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
