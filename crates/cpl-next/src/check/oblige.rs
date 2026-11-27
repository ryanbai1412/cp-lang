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
            self.obligations.push((f.targs[*i].clone(), r, span));
        }
    }
    /// Meets each bound whose type is known by now; `last` drops those never known.
    pub(super) fn solve(&mut self, last: bool) {
        loop {
            let pending = std::mem::take(&mut self.obligations);
            let before = pending.len();
            for (t, r, span) in pending {
                if self.unknown(&t) {
                    self.obligations.push((t, r, span));
                    continue;
                }
                let t = self.zonk(&t);
                match self.meet(&t, &r, span) {
                    Some(true) => {}
                    Some(false) => {
                        let tr = self.d.traits[r.tr.0].name.clone();
                        let shown = self.show(&t);
                        let record = matches!(&t, Ty::Struct(d, ..) if crate::records::is_record(&self.d.s(*d).name));
                        if record {
                            let msg = format!("slice: `{shown}` meeting `{tr}` by its fields");
                            self.err(span, msg);
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
                    None => self.obligations.push((t, r, span)),
                }
            }
            if self.obligations.len() >= before {
                break;
            }
        }
        if last {
            self.obligations.clear();
        }
    }
    /// Whether a pending bound is on the inference variable `v`: it, not a default, fixes `v`.
    pub(super) fn obliged(&self, v: usize) -> bool {
        (self.obligations.iter()).any(|(t, ..)| self.shallow(t) == Ty::Var(v))
    }
    /// Whether some part of `t` is still an inference variable.
    fn unknown(&self, t: &Ty) -> bool {
        match self.shallow(t) {
            Ty::Var(_) => true,
            Ty::Raw(e) | Ty::Ref(e) => self.unknown(&e),
            Ty::Struct(_, ts, _) => ts.iter().any(|t| self.unknown(t)),
            Ty::Fn(ps, r, _) => ps.iter().any(|t| self.unknown(t)) || self.unknown(&r),
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
            self.obligations
                .extend(needs.into_iter().map(|n| (t.clone(), n, span)));
            return Some(true);
        }
        if Some(r.tr) == self.d.domain_trait {
            return Some(matches!(t, Ty::Dom(_) | Ty::Error));
        }
        match t {
            Ty::Error => Some(true),
            Ty::Param(i) => {
                let tr = &self.d.traits[r.tr.0];
                let mut have = self.bound_traits(*i).into_iter().filter(|h| h.tr == r.tr);
                let fitting: Vec<TraitRef> = (have.clone())
                    .filter(|h| {
                        (tr.inputs(&h.args).iter().zip(tr.inputs(&r.args)))
                            .all(|(a, b)| self.fits(a, b))
                    })
                    .collect();
                let Some(h) = fitting.first().cloned().or_else(|| have.next()) else {
                    return Some(false);
                };
                if fitting.len() > 1 && tr.inputs(&r.args).iter().any(|a| self.unknown(a)) {
                    return None;
                }
                Some(h.args.iter().zip(&r.args).all(|(a, b)| self.unify(a, b)))
            }
            _ => {
                // The impl whose trait arguments fit too: `Int` has `Into<Char>` and `Into<U64>`.
                let want: Vec<Ty> = (self.d.traits[r.tr.0].inputs(&r.args).iter())
                    .map(|a| self.zonk(a))
                    .collect();
                let found: Vec<(usize, Vec<Ty>)> = (self.d.impls.iter().enumerate())
                    .filter(|(_, im)| im.trait_.tr == r.tr)
                    .filter_map(|(k, im)| Some((k, self.d.impl_args_for(im, t, &want)?)))
                    .collect();
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
                    .extend(needs.into_iter().map(|(t, b)| (t, b, span)));
                Some(args.iter().zip(&r.args).all(|(a, b)| self.unify(a, b)))
            }
        }
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
