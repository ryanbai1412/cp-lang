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

/// The name an impl is filed under: `impl<D: Domain> Tr for D` is every domain's, `domain`;
/// `impl<R: (to: V, ..)> Tr for R` every record's, `record`; `impl Tr for [V]S` every
/// table's, `array`.
pub(crate) fn impl_file_head(ty: &ast::Type, bounds: &[ast::Bound]) -> Option<String> {
    if let TypeKind::Array { .. } = &ty.kind {
        return Some("array".into());
    }
    if let TypeKind::Named { name, args } = &ty.kind {
        let domain = |t: &ast::Type| type_head(t).as_deref() == Some("Domain");
        let own = || bounds.iter().filter(|b| b.name.text == name.text);
        if args.is_empty() && own().any(|b| b.traits.iter().any(domain)) {
            return Some("domain".into());
        }
        if args.is_empty() && row_bound(ty, bounds).is_some() {
            return Some("record".into());
        }
    }
    impl_target_head(ty)
}

/// The row `R: (to: V, ..)` a bare impl target `R` is bounded by.
pub(crate) fn row_bound<'t>(ty: &ast::Type, bounds: &'t [ast::Bound]) -> Option<&'t ast::Type> {
    let TypeKind::Named { name, args } = &ty.kind else {
        return None;
    };
    if !args.is_empty() {
        return None;
    }
    let own = bounds.iter().filter(|b| b.name.text == name.text);
    own.flat_map(|b| &b.traits)
        .find(|t| matches!(t.kind, TypeKind::Row(_)))
}

/// The name the impls written for `ty` itself are filed under.
pub(crate) fn ty_impl_name(ty: &Ty) -> Option<&str> {
    match ty {
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
    }
}

/// The name the impls a value of `ty` can find by its shape are filed under.
pub(crate) fn shape_impl_key(ty: &Ty) -> Option<&'static str> {
    match ty {
        Ty::Array { .. } => Some("array"),
        Ty::Record(_) => Some("record"),
        _ => None,
    }
}

/// Whether an impl for the pattern `target` covers `ty`: a row needs each field it names,
/// a table its number of axes.
fn shape_fits(target: &Ty, ty: &Ty) -> bool {
    match (target, ty) {
        (Ty::Record(want), Ty::Record(got)) => want
            .iter()
            .all(|(n, _)| n == ".." || got.iter().any(|(g, _)| g == n)),
        (Ty::Array { index: want, .. }, Ty::Array { index: got, .. }) => want.len() == got.len(),
        _ => false,
    }
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

    /// The impl of `trait_name` a table or record has by its shape: `impl Tr for [V]List<A>`,
    /// or `impl<R: (to: V, ..)> Tr for R`.
    pub(crate) fn shape_impl(&mut self, ty: &Ty, trait_name: &str) -> Option<Arc<Def>> {
        let key = shape_impl_key(ty)?;
        let head =
            |t: &Option<String>| t.as_deref().and_then(|t| t.split('<').next()) == Some(trait_name);
        (self.env.impls(key).into_iter())
            .filter(|(t, _)| head(t))
            .map(|(_, d)| d)
            .find(|d| matches!(&**d, Def::Impl { target, .. } if shape_fits(target, ty)))
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
        self.impl_member_under("domain", pos, name)
    }

    /// `e.tgt` on a table or record: the member an impl for its shape gives it, as
    /// `impl<V, R: (to: V, ..)> Tg<V> for R` does every record with a `to`. Else `Unknown`.
    pub(super) fn shape_impl_member(&mut self, ty: &Ty, name: &str) -> Ty {
        match shape_impl_key(ty) {
            Some(key) => self.impl_member_under(key, ty, name),
            None => Ty::Unknown,
        }
    }

    fn impl_member_under(&mut self, key: &str, pos: &Ty, name: &str) -> Ty {
        for (trait_name, def) in self.env.impls(key) {
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
            if key != "domain" && !shape_fits(target, pos) {
                continue;
            }
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
