//! Associated types chosen by impl (R08.1.9, decision 456): a trait declares
//! `type Found`, each impl chooses it, and `P.Found` in a generic signature is
//! the choice of `P`'s impl once `P` is known.

use super::types::type_head;
use super::*;
use cpl_diag::msgs;
use cpl_syntax::ast::{self, Stmt, StmtKind};

/// The associated types a trait body declares, each with its bound's traits
/// and the default it writes (`type Item = Int`), if any.
pub(super) fn declared(body: &[Stmt]) -> Vec<(String, Vec<String>, Option<&ast::Type>)> {
    body.iter()
        .filter_map(|s| match &s.kind {
            StmtKind::TypeAlias {
                name, bounds, ty, ..
            } => Some((
                name.text.clone(),
                bounds.iter().filter_map(type_head).collect(),
                (!matches!(ty.kind, ast::TypeKind::Error)).then_some(ty),
            )),
            _ => None,
        })
        .collect()
}

impl Checker<'_> {
    /// `of.name` for a known `of`: the type the impl for `of` chooses (R08.1.9).
    pub(crate) fn project(&mut self, of: &Ty, name: &str) -> Option<Ty> {
        let key = match of {
            Ty::Named { name, .. } => name.clone(),
            Ty::Int | Ty::I128 | Ty::U64 | Ty::Float | Ty::Bool | Ty::Char | Ty::Str => {
                of.to_string()
            }
            Ty::Fn { .. } => "fn".into(),
            _ => return None,
        };
        for (implemented, def) in self.env.impls(&key) {
            let Def::Impl {
                target,
                assoc,
                trait_args,
                ..
            } = &*def
            else {
                continue;
            };
            let Some((_, chosen)) = assoc.iter().find(|(n, _)| n == name) else {
                continue;
            };
            let mut bindings = crate::ty::Bindings::default();
            target.unify(of, &mut bindings);
            // A fn item's own type gives the impl's generics through the bound's trait arguments.
            let tr = implemented.as_deref().and_then(|t| t.split('<').next());
            let given = self
                .item_args
                .iter()
                .rev()
                .find(|(i, t, _)| *i == key && Some(&**t) == tr);
            for (want, got) in trait_args.iter().zip(given.map_or(&[][..], |(.., a)| a)) {
                want.unify(got, &mut bindings);
            }
            if of.fits(&target.subst(&bindings)) {
                return Some(chosen.subst(&bindings));
            }
        }
        None
    }

    /// An impl's associated types with the trait's defaults for the ones it
    /// leaves out, seen with `Self` as the impl's target (decision 469).
    pub(super) fn with_assoc_defaults(
        &mut self,
        trait_ty: Option<&ast::Type>,
        self_ty: &Ty,
        mut assoc: Vec<(String, Ty)>,
    ) -> Vec<(String, Ty)> {
        let Some(Def::Trait {
            assoc: declared, ..
        }) = trait_ty
            .and_then(type_head)
            .and_then(|n| self.global(&n))
            .as_deref()
            .cloned()
        else {
            return assoc;
        };
        let mut on = crate::ty::Bindings::default();
        on.insert("Self".into(), self_ty.clone());
        for (name, _, default) in declared {
            if let Some(default) = default {
                if assoc.iter().all(|(n, _)| *n != name) {
                    assoc.push((name, default.subst(&on)));
                }
            }
        }
        assoc
    }

    /// The members impls for `of`'s shape give it (a fn type, a domain position, a table or
    /// record: `impl<V, R: (to: V, ..)> Tg<V> for R`), written in the impl or a default of its
    /// trait; a `@get` reads as its value.
    pub(super) fn shape_impl_members(&mut self, of: &Ty) -> Vec<(String, Ty)> {
        let mut out = Vec::new();
        let Some(key) = super::impl_head::ty_impl_name(of) else {
            return out;
        };
        for (trait_name, def) in self.env.impls(key) {
            let Def::Impl {
                target,
                methods,
                getters,
                ..
            } = &*def
            else {
                continue;
            };
            let mut bindings = crate::ty::Bindings::default();
            target.unify(of, &mut bindings);
            if !of.fits(&target.subst(&bindings)) {
                continue;
            }
            out.extend(methods.iter().map(|(m, t)| {
                let t = match t.subst(&bindings) {
                    Ty::Fn { ret, .. } if getters.contains(m) => *ret,
                    t => t,
                };
                (m.clone(), t)
            }));
            // What the impl leaves to its trait's defaults (`f.lift(x)`, `(+).laws`).
            let tr = trait_name.as_deref().and_then(|t| t.split('<').next());
            let Some(Def::Trait {
                methods, generics, ..
            }) = tr.and_then(|tr| self.type_def(tr)).as_deref().cloned()
            else {
                continue;
            };
            let inherited = self.trait_members(&methods, &generics, &def);
            out.extend(inherited.into_iter().map(|(m, t)| (m, t.subst(&bindings))));
        }
        out
    }

    pub(super) fn shape_impl_member(&mut self, of: &Ty, name: &str) -> Ty {
        let found = self
            .shape_impl_members(of)
            .into_iter()
            .find(|(m, _)| m == name);
        found.map_or(Ty::Unknown, |m| m.1)
    }

    /// Replaces each `M.Fold` whose `M` is now known with the type its impl chooses.
    pub(crate) fn resolve_assoc(&mut self, ty: &Ty) -> Ty {
        if !ty.has_assoc() {
            return ty.clone();
        }
        ty.map(&mut |t| match t {
            Ty::Assoc { of, name } => {
                let of = self.resolve_assoc(of);
                Some(match of {
                    Ty::Param(_) => Ty::Assoc {
                        of: Box::new(of),
                        name: name.clone(),
                    },
                    Ty::Unknown | Ty::Error => of,
                    _ => self.project(&of, name).unwrap_or(Ty::Unknown),
                })
            }
            _ => None,
        })
    }

    /// An impl chooses each associated type its trait declares, and only those (R08.1.9).
    pub(super) fn impl_chooses_assoc_types(
        &mut self,
        trait_ty: &ast::Type,
        target: &ast::Type,
        body: &[Stmt],
    ) {
        let Some(Def::Trait {
            assoc: declared, ..
        }) = type_head(trait_ty)
            .and_then(|n| self.global(&n))
            .as_deref()
            .cloned()
        else {
            return;
        };
        let assoc: Vec<String> = declared.iter().map(|(n, ..)| n.clone()).collect();
        let trait_text = self.quote(trait_ty.span);
        let chosen: Vec<&ast::Name> = body
            .iter()
            .filter_map(|s| match &s.kind {
                StmtKind::TypeAlias { name, .. } => Some(name),
                _ => None,
            })
            .collect();
        for name in &chosen {
            if !assoc.contains(&name.text) {
                if let Some(diag) = self.report(
                    name.span,
                    msgs::check::UNKNOWN_FIELD_DECLARES_NO_ASSOCIATED_TYPE,
                    &[&trait_text, &name.text],
                ) {
                    diag.help = Some(match assoc.is_empty() {
                        true => format!("`{trait_text}` declares no associated types"),
                        false => format!("it declares {}", super::expr::list_names(&assoc)),
                    });
                }
            }
        }
        for s in body {
            let StmtKind::TypeAlias { name, ty, .. } = &s.kind else {
                continue;
            };
            let Some((_, bounds, _)) = declared.iter().find(|(n, ..)| *n == name.text) else {
                continue;
            };
            let chosen = self.quietly(|c| c.resolve_type(ty));
            // An impl's own unbounded parameter (`type Arc = V` for any `V`)
            // has no traits to check yet.
            if matches!(&chosen, Ty::Param(p) if !self.param_bounds.iter().any(|(q, ts)| q == p && !ts.is_empty()))
            {
                continue;
            }
            for b in bounds {
                if self.implements_trait(&chosen, b) {
                    continue;
                }
                if let Some(diag) = self.report(
                    ty.span,
                    msgs::check::MISSING_IMPL_DOES_NOT_IMPLEMENT_WHICH,
                    &[&chosen, &b, &trait_text, &name.text],
                ) {
                    diag.help = Some(format!(
                        "choose a type that implements `{b}`: the trait declares `type {}: {b}`",
                        name.text
                    ));
                }
            }
        }
        // A defaulted associated type may be left out (decision 469).
        let missing: Vec<String> = declared
            .iter()
            .filter(|(a, _, default)| default.is_none() && chosen.iter().all(|c| &c.text != a))
            .map(|(a, ..)| a.clone())
            .collect();
        if !missing.is_empty() {
            let span = trait_ty.span.to(target.span);
            if let Some(diag) = self.report(
                span,
                msgs::check::MISSING_IMPL_IMPL_FOR_DOES_NOT,
                &[
                    &trait_text,
                    &self.quote(target.span),
                    &super::expr::list_names(&missing),
                ],
            ) {
                diag.help = Some(format!(
                    "write `type {} = ...` in the impl: each impl chooses its trait's associated types",
                    missing[0]
                ));
            }
        }
    }

    /// The fns a trait gives an impl's target: the ones the impl does not
    /// write itself, seeing `Self`, `Self.Item` and the trait's parameters as the impl's.
    pub(super) fn trait_members(
        &mut self,
        methods: &[crate::def::TraitMethod],
        generics: &[String],
        def: &Def,
    ) -> Vec<(String, Ty)> {
        let Def::Impl {
            target,
            methods: own,
            trait_args,
            ..
        } = def
        else {
            return methods
                .iter()
                .map(|m| (m.name.clone(), read_as(m, m.ty.clone())))
                .collect();
        };
        let mut on = crate::ty::Bindings::default();
        on.insert("Self".into(), target.clone());
        for (g, a) in generics.iter().zip(trait_args) {
            on.insert(g.clone(), a.clone());
        }
        methods
            .iter()
            .filter(|m| !own.iter().any(|(name, _)| *name == m.name))
            .map(|m| {
                let ty = self.resolve_assoc(&m.ty.subst(&on));
                (m.name.clone(), read_as(m, ty))
            })
            .collect()
    }
}

/// What reading the trait fn `m` as a member gives: a `@get` fn's value, else the fn.
fn read_as(m: &crate::def::TraitMethod, ty: Ty) -> Ty {
    match ty {
        Ty::Fn { ret, .. } if m.getter => *ret,
        ty => ty,
    }
}
