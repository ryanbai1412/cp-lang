//! Whether a type implements a trait: written impls, bounds, and what
//! built-in types and structural shapes give.

use super::*;

impl Checker<'_> {
    /// True when trait `sub` is `want` or transitively inherits from `want`.
    pub(crate) fn trait_implies(&mut self, sub: &str, want: &str) -> bool {
        let mut stack = vec![sub.to_string()];
        let mut seen = Vec::new();
        while let Some(cur) = stack.pop() {
            if cur == want {
                return true;
            }
            if seen.iter().any(|s| s == &cur) {
                continue;
            }
            seen.push(cur.clone());
            match cur.as_str() {
                "Ord" => stack.push("Eq".into()),
                "IndexedMut" => stack.push("Indexed".into()),
                "Indexed" => {
                    stack.push("Positional".into());
                    stack.push("Bidir".into());
                    stack.push("Keyed".into());
                }
                "Positional" => stack.push("Sized".into()),
                "Sized" | "Bidir" | "Keyed" => stack.push("Seq".into()),
                _ => {}
            }
            if let Some(Def::Trait { supers, .. }) = self.global(&cur).as_deref() {
                stack.extend(supers.iter().cloned());
            }
        }
        false
    }

    /// True when the in-scope type parameter `param` has a bound implying `trait_name`.
    pub(crate) fn param_has_bound(&mut self, param: &str, trait_name: &str) -> bool {
        let traits: Vec<String> = self
            .param_bounds
            .iter()
            .filter(|(p, _)| p == param)
            .flat_map(|(_, ts)| ts.iter().cloned())
            .collect();
        traits.iter().any(|t| self.trait_implies(t, trait_name))
    }

    /// True when `ty` implements `trait_name`, via explicit `impl`, structural
    /// derivation (`Eq`/`Ord`/`Hash`/`Print`), built-in sequence capability, or
    /// type-parameter bounds.
    pub(crate) fn implements_trait(&mut self, ty: &Ty, trait_name: &str) -> bool {
        let mut seen = Vec::new();
        self.implements_trait_inner(ty, trait_name, &mut seen)
    }

    pub(super) fn implements_trait_inner(
        &mut self,
        ty: &Ty,
        trait_name: &str,
        seen: &mut Vec<String>,
    ) -> bool {
        match ty {
            Ty::Error | Ty::Unknown => return true,
            Ty::Param(p) => {
                if !self.generics.iter().any(|g| g == p) {
                    return true;
                }
                return self.param_has_bound(p, trait_name);
            }
            _ => {}
        }
        if let Some(Def::Trait {
            supers,
            alias: true,
            ..
        }) = self.type_def(trait_name).as_deref().cloned()
        {
            return supers
                .iter()
                .all(|s| self.implements_trait_inner(ty, s, seen));
        }
        let impl_key = match ty {
            Ty::Named { name, .. } => Some(name.as_str()),
            Ty::Int => Some("Int"),
            Ty::I128 => Some("I128"),
            Ty::U64 => Some("U64"),
            Ty::Float => Some("F64"),
            Ty::Bool => Some("Bool"),
            Ty::Char => Some("Char"),
            Ty::Str => Some("Str"),
            Ty::Fn { .. } => Some("fn"),
            Ty::Pos(_) | Ty::Domain(_) => Some("domain"),
            _ => None,
        };
        if impl_key.is_some_and(|name| self.has_impl_for(ty, name, trait_name, seen))
            || self.rev_has(ty, trait_name, seen)
        {
            return true;
        }
        match trait_name {
            "Eq" | "Ord" | "Hash" => self.derives_structural(ty, trait_name, seen),
            "Show" => self.derives_print(ty, false, seen),
            "Seq" => ty.is_seq() || self.walks_by_each(ty),
            "Sized" => ty.is_sized(),
            "Indexed" => ty.is_indexed(),
            "Positional" => ty.rev_source().unwrap_or(ty).is_indexed(),
            super::views::INDEXED_BY_WALK => ty.rev_source().unwrap_or(ty).is_indexed(),
            "Bidir" => ty.is_bidir(),
            "Keyed" => ty.is_keyed(),
            "IndexedMut" => ty.is_indexed_mut(),
            "Contains" => match ty {
                Ty::Str | Ty::Domain(_) => true,
                Ty::Named { name, .. } => matches!(
                    name.as_str(),
                    "Set"
                        | "HashSet"
                        | "SortedSet"
                        | "SortedBag"
                        | "Map"
                        | "HashMap"
                        | "SortedMap"
                        | "Bitset"
                        | "Range"
                        | "Counter"
                        | "Bag"
                ),
                _ => false,
            },
            "Add" => ty.is_numeric() || matches!(ty, Ty::Str),
            "Sub" | "Mul" | "Div" | "Rem" | "Pow" | "Neg" | "Num" => ty.is_numeric(),
            "Index" => ty.is_int() || matches!(ty, Ty::Pos(_) | Ty::Char),
            "Domain" => match ty {
                Ty::Pos(_) | Ty::Domain(_) => true,
                Ty::Named { name, .. } => self.has_impl(name, "Index"),
                _ => false,
            },
            "Target" => match ty {
                Ty::Pos(_) | Ty::Int => true,
                Ty::Record(fields) => fields.iter().any(|(n, _)| n == "to"),
                _ => false,
            },
            "Next" | "WeightedNext" | "EdgedNext" => matches!(ty, Ty::Array { .. } | Ty::Fn { .. }),
            _ => match ty {
                Ty::Record(fields) => self.meets_by_shape(fields, trait_name),
                _ => false,
            },
        }
    }

    /// R04.2.10: the factor positions one array axis stands for, `[P]T` with
    /// `domain P = (R, C)` or `type P = (R, C)`.
    pub(crate) fn axis_factors(&mut self, axis: &Ty) -> Option<Vec<Ty>> {
        match axis {
            Ty::Pos(d) => Some(
                self.product_factors(d)?
                    .into_iter()
                    .map(|(_, t)| t)
                    .collect(),
            ),
            Ty::Tuple(items) if items.len() > 1 => Some(items.clone()),
            _ => None,
        }
    }

    /// R08.1.7: a record meets a trait when a field meets each member the trait leaves without a
    /// body, a field `w: W` or a bodiless `@get fn w(self) -> W` alike, with its type.
    pub(crate) fn meets_by_shape(&mut self, fields: &[(String, Ty)], trait_name: &str) -> bool {
        let Some(Def::Trait {
            methods,
            fields: wanted,
            ..
        }) = self.global(trait_name).as_deref().cloned()
        else {
            return true;
        };
        let getters = methods
            .into_iter()
            .filter(|m| m.required)
            .map(|m| match m.ty {
                Ty::Fn { params, ret } if params.len() <= 1 => (m.name, *ret),
                _ => (m.name, Ty::Unknown),
            });
        let wanted: Vec<(String, Ty)> = getters.chain(wanted).collect();
        wanted.iter().all(|(name, want)| {
            fields.iter().any(|(f, got)| {
                f == name && (matches!(want, Ty::Param(_) | Ty::Unknown) || got.fits(want))
            })
        })
    }

    fn derives_structural(&mut self, ty: &Ty, trait_name: &str, seen: &mut Vec<String>) -> bool {
        match ty {
            Ty::Error | Ty::Unknown | Ty::Const(_) => true,
            Ty::Param(p) => self.param_has_bound(p, trait_name),
            // Its trait's `type X: Bound` is not tracked yet, so it is taken as met.
            Ty::Assoc { .. } => true,
            Ty::Int
            | Ty::I128
            | Ty::U64
            | Ty::Bool
            | Ty::Char
            | Ty::Str
            | Ty::Unit
            | Ty::Range { .. }
            | Ty::Pos(_)
            | Ty::Domain(_) => true,
            Ty::Float => trait_name != "Hash",
            Ty::Fn { .. } => false,
            Ty::Optional(inner) => self.implements_trait_inner(inner, trait_name, seen),
            Ty::Tuple(items) => items
                .iter()
                .all(|t| self.implements_trait_inner(t, trait_name, seen)),
            Ty::Record(fields) => fields
                .iter()
                .all(|(_, t)| self.implements_trait_inner(t, trait_name, seen)),
            Ty::Array { elem, .. } => self.implements_trait_inner(elem, trait_name, seen),
            Ty::Named { name, args } => {
                if seen.iter().any(|s| s == name) {
                    return true;
                }
                if trait_name == "Ord" && matches!(name.as_str(), "HashMap" | "HashSet") {
                    return false;
                }
                // A cut or range compares by position, whatever its domain
                // holds, as `Ty::Pos` and `Ty::Range` do.
                if matches!(name.as_str(), "Cut" | "Range") {
                    return true;
                }
                match self.type_def(name).as_deref().cloned() {
                    Some(Def::Struct {
                        generics, fields, ..
                    }) => {
                        seen.push(name.clone());
                        let mut bindings = crate::ty::Bindings::default();
                        for (g, a) in generics.iter().zip(args) {
                            bindings.insert(g.clone(), a.clone());
                        }
                        let ok = fields.iter().all(|f| {
                            let fty = f.ty.subst(&bindings);
                            self.implements_trait_inner(&fty, trait_name, seen)
                        });
                        seen.pop();
                        ok
                    }
                    Some(Def::Enum { variants, .. }) => {
                        seen.push(name.clone());
                        let ok = variants.iter().all(|(_, fs)| {
                            fs.iter()
                                .all(|fty| self.implements_trait_inner(fty, trait_name, seen))
                        });
                        seen.pop();
                        ok
                    }
                    _ => args
                        .iter()
                        .all(|a| self.implements_trait_inner(a, trait_name, seen)),
                }
            }
            Ty::RowWithout(_, inner, _) => self.implements_trait_inner(inner, trait_name, seen),
            Ty::TypeName(_) => false,
        }
    }

    /// Is there a written `impl trait_name for name`?
    /// `has_impl` for `ty` itself: a generic impl counts only when `ty` meets its head's
    /// bounds (`impl Indexed<D, (D, T)> for Items<S: Indexed<D, T>>` is no impl for an
    /// `Items<Rev<List<Int>>>`).
    fn has_impl_for(
        &mut self,
        ty: &Ty,
        name: &str,
        trait_name: &str,
        seen: &mut Vec<String>,
    ) -> bool {
        if self
            .global(&crate::def::impl_name(Some(trait_name), name))
            .is_some()
        {
            return true;
        }
        let impls: Vec<Arc<Def>> = self
            .env
            .impls(name)
            .into_iter()
            .filter(|(t, _)| {
                t.as_deref()
                    .is_some_and(|t| t.split('<').next().unwrap_or(t) == trait_name)
            })
            .map(|(_, def)| def)
            .collect();
        impls.iter().any(|def| {
            let Def::Impl {
                target, bound_args, ..
            } = &**def
            else {
                return true;
            };
            let mut bindings = crate::ty::Bindings::default();
            target.unify(ty, &mut bindings);
            bound_args.iter().all(|(param, bound, _)| {
                let Some(actual) = bindings.get(param).cloned() else {
                    return true;
                };
                let bound = bound.split('<').next().unwrap_or(bound).to_string();
                matches!(actual, Ty::Param(_) | Ty::Unknown | Ty::Error)
                    || self.implements_trait_inner(&actual, &bound, seen)
            })
        })
    }

    pub(super) fn has_impl(&mut self, name: &str, trait_name: &str) -> bool {
        self.global(&crate::def::impl_name(Some(trait_name), name))
            .is_some()
            || self
                .env
                .impls(name)
                .into_iter()
                .filter_map(|(t, _)| t)
                .any(|t| t.split('<').next().unwrap_or(&t) == trait_name)
    }

    /// True when `ty` can be passed to `print(...)`: a written `impl Show`
    /// is its print hook, else it prints field by field.
    /// When `allow_none` is true (`print(x, none: "...")`), `?T` is printable if `T` is.
    pub(super) fn derives_print(&mut self, ty: &Ty, none: bool, seen: &mut Vec<String>) -> bool {
        let allow_none = none;
        match ty {
            Ty::Error | Ty::Unknown | Ty::Const(_) => true,
            Ty::Param(p) => self.param_has_bound(p, "Show"),
            // Its trait's `type X: Bound` is not tracked yet, so it is taken as met.
            Ty::Assoc { .. } => true,
            Ty::Int
            | Ty::I128
            | Ty::U64
            | Ty::Float
            | Ty::Char
            | Ty::Str
            | Ty::Range { .. }
            | Ty::Pos(_)
            | Ty::Unit => true,
            Ty::Bool => false,
            Ty::Optional(inner) => {
                if matches!(inner.as_ref(), Ty::Pos(_)) {
                    return true;
                }
                allow_none && self.derives_print(inner, allow_none, seen)
            }
            Ty::Fn { params, ret } => {
                params.is_empty() && self.derives_print(ret, allow_none, seen)
            }
            Ty::Tuple(items) => items
                .iter()
                .all(|t| self.derives_print(t, allow_none, seen)),
            Ty::Record(fields) => fields
                .iter()
                .all(|(_, t)| self.derives_print(t, allow_none, seen)),
            Ty::Array { elem, .. } => self.derives_print(elem, allow_none, seen),
            Ty::Named { name, args } => {
                if seen.iter().any(|s| s == name) || self.has_impl(name, "Show") {
                    return true;
                }
                if matches!(name.as_str(), "HashMap" | "HashSet") {
                    return false;
                }
                if name == "List" && args.len() == 1 {
                    return self.derives_print(&args[0], allow_none, seen);
                }
                match self.type_def(name).as_deref().cloned() {
                    Some(Def::Struct {
                        generics, fields, ..
                    }) => {
                        seen.push(name.clone());
                        let mut bindings = crate::ty::Bindings::default();
                        for (g, a) in generics.iter().zip(args) {
                            bindings.insert(g.clone(), a.clone());
                        }
                        let ok = fields.iter().all(|f| {
                            let fty = f.ty.subst(&bindings);
                            self.derives_print(&fty, allow_none, seen)
                        });
                        seen.pop();
                        ok
                    }
                    Some(Def::Enum {
                        variants, tagged, ..
                    }) => {
                        if tagged {
                            return true;
                        }
                        seen.push(name.clone());
                        let ok = variants.iter().all(|(_, fs)| {
                            fs.iter()
                                .all(|fty| self.derives_print(fty, allow_none, seen))
                        });
                        seen.pop();
                        ok
                    }
                    _ => true,
                }
            }
            Ty::RowWithout(_, inner, _) => self.derives_print(inner, allow_none, seen),
            Ty::Domain(_) | Ty::TypeName(_) => true,
        }
    }
}
