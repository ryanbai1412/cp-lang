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
        TypeKind::ItemOf(item) => item.fn_item_type(),
        _ => type_head(ty),
    }
}

impl Checker<'_> {
    /// Whether `n` names a fn item, not a local or another global.
    pub(crate) fn names_fn_item(&mut self, n: &str) -> bool {
        self.local(n).is_none() && self.global_is(n, |d| matches!(d, Def::Fn { .. }))
    }

    /// `type_of(f)`: the own type of the fn item `f` names (R08.1.10). A
    /// name nothing defines is E0201; a `let`, a parameter or any other
    /// value has no item behind it, E0324.
    pub(crate) fn item_of_type(&mut self, item: &ast::Expr) -> Ty {
        let mut inner = item;
        while let ast::ExprKind::Paren(e) = &inner.kind {
            inner = e;
        }
        let what = match &inner.kind {
            ast::ExprKind::Name(n) => match self.local(n) {
                Some(local) => match local.keyword {
                    "" | "param" => Some(("a parameter".to_string(), local.ty.clone())),
                    kw => Some((format!("a `{kw}`"), local.ty.clone())),
                },
                None => match self.global(n).as_deref() {
                    Some(Def::Fn { .. }) => None,
                    Some(Def::Value { .. }) => {
                        let ty = self.name_ty(n, inner.span);
                        Some(("a `let`".to_string(), ty))
                    }
                    Some(_) => Some(("a type".to_string(), Ty::Unknown)),
                    None => match self.name_ty(n, inner.span) {
                        Ty::Unknown | Ty::Error => return Ty::Unknown,
                        Ty::Fn { .. } => None,
                        Ty::TypeName(_) | Ty::Domain(_) => {
                            Some(("a type".to_string(), Ty::Unknown))
                        }
                        ty => Some(("a value".to_string(), ty)),
                    },
                },
            },
            _ if inner.fn_item_key().is_some() => None,
            _ => Some(("an expression".to_string(), Ty::Unknown)),
        };
        let Some((what, ty)) = what else {
            return match item.fn_item_type() {
                Some(name) => Ty::Named {
                    name,
                    args: Vec::new(),
                },
                None => Ty::Unknown,
            };
        };
        let text = self.quote(inner.span);
        if let Some(diag) = self.report(inner.span, msgs::check::NOT_A_FN_ITEM, &[&text, &what]) {
            diag.help = Some(match ty {
                Ty::Fn { .. } => format!("declare it with `fn`, or write its fn type `{ty}`"),
                _ => "name a `fn`, or an operator written bare".to_string(),
            });
        }
        Ty::Error
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
                cpl_diag::msgs::check::IMPL_ADDS_INHERENT_METHODS.at(&[&shown, &shown])
            }
            Some(head) if !self.env.declared().contains(&head) => {
                cpl_diag::msgs::check::IMPL_ADDS_INHERENT_METHODS_2.at(&[&shown, &head])
            }
            _ => return,
        };
        if let Some(diag) = self.report_said(target.span, message) {
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
                    msgs::check::MISSING_IMPL_IMPL_FOR_REQUIRES_IMPL,
                    &[&tr_text, &target_text, &sup, &target_text],
                );
            }
        }
        self.param_bounds.truncate(outer);
    }

    /// Whether an impl of `trait_name` for `name` covers `ty`: its target's generics, bound to
    /// `ty`'s arguments, meet the bounds the impl head writes (`impl Sized for Rev<S: Sized>`).
    pub(super) fn impl_holds(
        &mut self,
        name: &str,
        ty: &Ty,
        trait_name: &str,
        seen: &mut Vec<String>,
    ) -> bool {
        let written = self.global(&crate::def::impl_name(Some(trait_name), name));
        let listed = (self.env.impls(name).into_iter())
            .filter(|(t, _)| t.as_deref().and_then(|t| t.split('<').next()) == Some(trait_name))
            .map(|(_, d)| d);
        written.into_iter().chain(listed).any(|def| {
            let Def::Impl {
                target, bound_args, ..
            } = &*def
            else {
                return true;
            };
            let mut on = crate::ty::Bindings::default();
            target.unify(ty, &mut on);
            bound_args.iter().all(|(param, bound, _)| {
                on.get(param)
                    .cloned()
                    .is_none_or(|actual| self.implements_trait_inner(&actual, bound, seen))
            })
        })
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

    /// R08.1.7: a record or tuple meets a trait by its shape, so it has the trait's default
    /// methods as a type with an empty impl would: `(3, 4).rev()` is `Reversible`'s.
    pub(super) fn shape_default_member(&mut self, ty: &Ty, name: &str) -> Option<Ty> {
        let fields = match ty {
            Ty::Record(fields) => fields.clone(),
            Ty::Tuple(items) => super::types::positional(items),
            _ => return None,
        };
        for trait_name in self.env.names() {
            let Some(Def::Trait {
                methods,
                generics,
                alias: false,
                ..
            }) = self.global(&trait_name).as_deref().cloned()
            else {
                continue;
            };
            if !generics.is_empty() {
                continue;
            }
            let Some(method) = methods.iter().find(|m| m.name == name && !m.required) else {
                continue;
            };
            if !self.meets_by_shape(&fields, &trait_name) {
                continue;
            }
            let mut on = crate::ty::Bindings::default();
            on.insert("Self".into(), ty.clone());
            return Some(method.ty.subst(&on));
        }
        None
    }
}
