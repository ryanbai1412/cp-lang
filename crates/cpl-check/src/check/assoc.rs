//! Associated types chosen by impl (R08.1.9, decision 456): a trait declares
//! `type Found`, each impl chooses it, and `P.Found` in a generic signature is
//! the choice of `P`'s impl once `P` is known.

use super::types::type_head;
use super::*;
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
        for (_, def) in self.env.impls(&key) {
            let Def::Impl { target, assoc, .. } = &*def else {
                continue;
            };
            let Some((_, chosen)) = assoc.iter().find(|(n, _)| n == name) else {
                continue;
            };
            let mut bindings = crate::ty::Bindings::default();
            target.unify(of, &mut bindings);
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

    /// The methods impls for fn types give a fn value of type `of`.
    pub(super) fn fn_impl_members(&mut self, of: &Ty) -> Vec<(String, Ty)> {
        let mut out = Vec::new();
        for (_, def) in self.env.impls("fn") {
            let Def::Impl {
                target, methods, ..
            } = &*def
            else {
                continue;
            };
            let mut bindings = crate::ty::Bindings::default();
            target.unify(of, &mut bindings);
            if of.fits(&target.subst(&bindings)) {
                out.extend(methods.iter().map(|(m, t)| (m.clone(), t.subst(&bindings))));
            }
        }
        out
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
                let message = format!("`{trait_text}` declares no associated type `{}`", name.text);
                if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
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
                let message = format!(
                    "`{chosen}` does not implement `{b}`, which `{trait_text}` requires of `{}`",
                    name.text
                );
                if let Some(diag) = self.error(codes::MISSING_IMPL, ty.span, message) {
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
            let message = format!(
                "`impl {trait_text} for {}` does not choose {}",
                self.quote(target.span),
                super::expr::list_names(&missing)
            );
            let span = trait_ty.span.to(target.span);
            if let Some(diag) = self.error(codes::MISSING_IMPL, span, message) {
                diag.help = Some(format!(
                    "write `type {} = ...` in the impl: each impl chooses its trait's associated types",
                    missing[0]
                ));
            }
        }
    }

    /// The fns a trait gives an impl's target: the ones the impl does not
    /// write itself, seeing `Self` and `Self.Item` as the impl's.
    pub(super) fn trait_members(
        &mut self,
        methods: &[crate::def::TraitMethod],
        def: &Def,
    ) -> Vec<(String, Ty)> {
        let Def::Impl {
            target,
            methods: own,
            ..
        } = def
        else {
            return methods
                .iter()
                .map(|m| (m.name.clone(), m.ty.clone()))
                .collect();
        };
        let mut on = crate::ty::Bindings::default();
        on.insert("Self".into(), target.clone());
        methods
            .iter()
            .filter(|m| !own.iter().any(|(name, _)| *name == m.name))
            .map(|m| (m.name.clone(), self.resolve_assoc(&m.ty.subst(&on))))
            .collect()
    }
}
