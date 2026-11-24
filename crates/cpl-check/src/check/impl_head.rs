//! What an impl is for: the name it is filed under and the type parameters
//! its header leaves open.

use super::types::type_head;
use super::*;
use cpl_diag::msgs;
use cpl_syntax::ast::{self, TypeKind};

/// The name an impl for `ty` is filed under: its head, or `fn` for every fn type.
pub(crate) fn impl_target_head(ty: &ast::Type) -> Option<String> {
    match &ty.kind {
        TypeKind::Fn { .. } => Some("fn".into()),
        _ => type_head(ty),
    }
}

/// The name an impl is filed under: `impl<D: Domain> Tr for D` is every domain's, `domain`.
pub(crate) fn impl_file_head(ty: &ast::Type, bounds: &[ast::Bound]) -> Option<String> {
    if let TypeKind::Named { name, args } = &ty.kind {
        let domain = |t: &ast::Type| type_head(t).as_deref() == Some("Domain");
        if args.is_empty()
            && bounds
                .iter()
                .any(|b| b.name.text == name.text && b.traits.iter().any(domain))
        {
            return Some("domain".into());
        }
    }
    impl_target_head(ty)
}

impl<'a> Checker<'a> {
    /// The type an impl is for, with the type parameters its header names:
    /// `impl Add for Mod<P>` is generic over `P`, and `impl<T: Add> Add for ?T`
    /// is generic over `T`.
    /// Names left open by the trait's arguments or a bound count too: `T` in
    /// `impl Seq<T> for Wrap<S: Seq<T>>`.
    pub(crate) fn impl_generics(
        &mut self,
        trait_name: Option<&ast::Type>,
        target: &ast::Type,
        bounds: &[ast::Bound],
    ) -> Vec<String> {
        let mut out = Vec::new();
        for b in bounds {
            if !out.contains(&b.name.text) {
                out.push(b.name.text.clone());
            }
        }
        self.collect_impl_generics(target, &mut out);
        let traits = bounds.iter().flat_map(|b| &b.traits).chain(trait_name);
        for arg in traits.flat_map(|t| match &t.kind {
            TypeKind::Named { args, .. } => &args[..],
            _ => &[],
        }) {
            self.collect_impl_generics(arg, &mut out);
        }
        out
    }

    fn collect_impl_generics(&mut self, ty: &ast::Type, out: &mut Vec<String>) {
        match &ty.kind {
            TypeKind::Named { name, args } => {
                if args.is_empty()
                    && self.local(&name.text).is_none()
                    && self.global(&name.text).is_none()
                    && prelude::ty(&name.text).is_none()
                    && !out.contains(&name.text)
                {
                    out.push(name.text.clone());
                }
                for arg in args {
                    self.collect_impl_generics(arg, out);
                }
            }
            TypeKind::Optional(inner) | TypeKind::Mut(inner) => {
                self.collect_impl_generics(inner, out)
            }
            TypeKind::Tuple(items) => {
                for item in items {
                    self.collect_impl_generics(item, out);
                }
            }
            TypeKind::Array { index, elem } => {
                for i in index {
                    self.collect_impl_generics(i, out);
                }
                self.collect_impl_generics(elem, out);
            }
            TypeKind::Fn { params, ret } => {
                for p in params {
                    self.collect_impl_generics(p, out);
                }
                self.collect_impl_generics(ret, out);
            }
            _ => {}
        }
    }

    /// R08.1.3, R08.5.2: inherent methods live with their type, so
    /// `impl m.P:` and `impl Range<Int>:` (a type declared elsewhere)
    /// are errors, not blocks no call reaches.
    pub(super) fn impl_home(
        &mut self,
        trait_name: Option<&cpl_syntax::ast::Type>,
        target: &cpl_syntax::ast::Type,
        self_ty: &Ty,
    ) {
        if trait_name.is_some() || matches!(self_ty, Ty::Param(_) | Ty::Unknown | Ty::Error) {
            return;
        }
        let shown = self.quote(target.span);
        let message = match type_head(target) {
            _ if shown.contains('.') => {
                format!("`impl {shown}:` adds inherent methods outside `{shown}`'s module")
            }
            Some(head) if !self.env.declared().contains(&head) => format!(
                "`impl {shown}:` adds inherent methods to `{head}`, which is declared outside this module"
            ),
            _ => return,
        };
        if let Some(diag) = self.error(codes::UNSUPPORTED, target.span, message) {
            diag.help = Some(format!(
                "move the methods into the type's module, or declare a trait and `impl` it for `{shown}`"
            ));
        }
    }
}

impl Checker<'_> {
    /// R08.1.3: `impl Sub for T` requires `impl Super for T`, with the impl's own `bounds` in scope.
    pub(crate) fn impl_supertraits(
        &mut self,
        tr: &ast::Type,
        target: &ast::Type,
        bounds: &[ast::Bound],
    ) {
        let TypeKind::Named { name, .. } = &tr.kind else {
            return;
        };
        // An alias has no impl of its own: that is E0501, not a missing supertrait.
        let Some(Def::Trait {
            supers,
            alias: false,
            ..
        }) = self.global(&name.text).as_deref().cloned()
        else {
            return;
        };
        let outer = self.param_bounds.len();
        self.param_bounds
            .extend(super::types::extract_bounds(bounds));
        let target_ty = self.resolve_type(target);
        for sup in supers {
            if !self.implements_trait(&target_ty, &sup) {
                let (tr_text, target_text) = (self.quote(tr.span), self.quote(target.span));
                self.report(
                    tr.span,
                    msgs::check::MISSING_IMPL_IMPL_FOR_REQUIRES_IMPL.at(&[
                        &tr_text,
                        &target_text,
                        &sup,
                        &target_text,
                    ]),
                );
            }
        }
        self.param_bounds.truncate(outer);
    }

    /// A `.rev` view walks, sizes and reads by position as its source does (R07.3.5).
    pub(super) fn rev_has(&mut self, ty: &Ty, trait_name: &str, seen: &mut Vec<String>) -> bool {
        let need = match trait_name {
            "Seq" | "Bidir" => "Bidir",
            "Sized" => "Sized",
            "Positional" => "Indexed",
            _ => return false,
        };
        ty.rev_source()
            .is_some_and(|src| self.implements_trait_inner(&src.clone(), need, seen))
    }
}

impl Checker<'_> {
    /// `v.tk` on a domain position: the member a blanket `impl<D: Domain> Tr for D` gives it,
    /// written in the impl or a default of `Tr`; a `@get` reads as its value. Else `Unknown`.
    pub(super) fn domain_impl_member(&mut self, pos: &Ty, name: &str) -> Ty {
        for (trait_name, def) in self.env.impls("domain") {
            let Def::Impl {
                methods,
                getters,
                target,
                ..
            } = &*def
            else {
                continue;
            };
            let own = methods.iter().find(|(m, _)| m == name).map(|m| m.1.clone());
            let found = own.or_else(|| {
                let tr = trait_name.as_deref()?.split('<').next()?.to_string();
                let Def::Trait {
                    methods, generics, ..
                } = &*self.type_def(&tr)?
                else {
                    return None;
                };
                let members = self.trait_members(&methods.clone(), &generics.clone(), &def);
                members.into_iter().find(|(m, _)| m == name).map(|m| m.1)
            });
            let Some(ty) = found else {
                continue;
            };
            let mut on = crate::ty::Bindings::default();
            target.unify(pos, &mut on);
            return match ty.subst(&on) {
                Ty::Fn { ret, .. } if getters.iter().any(|g| g == name) => *ret,
                ty => ty,
            };
        }
        Ty::Unknown
    }
}
