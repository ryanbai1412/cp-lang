//! A parameter typed by a trait makes its fn generic in that argument's
//! type (R08.2.1): `fn total(s: Seq<Int>)` is checked as
//! `fn total<S: Seq<Int>>(s: S)`, for a std trait and a user one alike.

use std::borrow::Cow;

use super::*;
use cpl_diag::msgs;
use cpl_syntax::ast;

impl Checker<'_> {
    /// `f` with each trait-typed parameter turned into a type parameter of
    /// its own, bounded by the trait as written; `f` itself when it has none.
    pub(crate) fn trait_params_generic<'f>(&mut self, f: &'f ast::FnDecl) -> Cow<'f, ast::FnDecl> {
        let mut out: Option<ast::FnDecl> = None;
        for (i, p) in f.params.iter().enumerate() {
            let Some(
                ty @ ast::Type {
                    kind: ast::TypeKind::Named { name, .. },
                    ..
                },
            ) = &p.ty
            else {
                continue;
            };
            let declared = |g: &ast::Name| g.text == name.text;
            if f.generics.iter().any(declared) || self.generics.contains(&name.text) {
                continue;
            }
            if !matches!(self.global(&name.text).as_deref(), Some(Def::Trait { .. })) {
                continue;
            }
            let decl = out.get_or_insert_with(|| f.clone());
            let mut text = written(ty);
            if decl.generics.iter().any(|g| g.text == text) {
                text = format!("{text} {}", p.name.text);
            }
            let generic = ast::Name {
                text,
                span: name.span,
            };
            decl.bounds.push(ast::Bound {
                name: generic.clone(),
                traits: vec![ty.clone()],
            });
            decl.params[i].ty = Some(ast::Type {
                kind: ast::TypeKind::Named {
                    name: generic.clone(),
                    args: Vec::new(),
                },
                span: ty.span,
            });
            decl.generics.push(generic);
        }
        out.map_or(Cow::Borrowed(f), Cow::Owned)
    }
}

/// `Seq<Int>` as written, the name its type parameter shows in messages.
fn written(ty: &ast::Type) -> String {
    match &ty.kind {
        ast::TypeKind::Named { name, args } if args.is_empty() => name.text.clone(),
        ast::TypeKind::Named { name, args } => {
            let args: Vec<String> = args.iter().map(written).collect();
            format!("{}<{}>", name.text, args.join(", "))
        }
        _ => "_".to_string(),
    }
}

impl Checker<'_> {
    /// `Counter(0)` where `new` takes `xs: Seq<K>`: each bound of the
    /// method's type parameters holds for what the call binds it to, checked
    /// at the call as a free fn's are, so a type that misses one never
    /// reaches the body. A parameter in `params` typed by the trait reads as
    /// that parameter's type mismatch, at its argument.
    pub(crate) fn method_bounds_hold(
        &mut self,
        callee: &str,
        (bounds, params): (&[(String, String)], &[crate::ty::Param]),
        bindings: &crate::ty::Bindings,
        (args, call): (&[ast::Arg], &ast::Expr),
    ) {
        for (param, trait_name) in bounds {
            let Some(ty) = bindings.get(param) else {
                continue;
            };
            let ty = self.settle(ty);
            if ty.has_vague() || self.implements_trait(&ty, trait_name) {
                continue;
            }
            let typed = Ty::Param(param.clone());
            let written = param == trait_name || param.starts_with(&format!("{trait_name}<"));
            let slot = params.iter().find(|p| written && p.ty == typed);
            let mut positional = params.iter().filter(|p| !p.named_only);
            let arg = slot.and_then(|slot| {
                args.iter().find(|a| match &a.name {
                    Some(n) => n.text == slot.name,
                    None => positional.next().is_some_and(|p| p.name == slot.name),
                })
            });
            if let (Some(slot), Some(arg)) = (slot, arg) {
                let shown = self.quote(arg.value.span);
                self.report(
                    arg.value.span,
                    msgs::check::BUT_2,
                    &[
                        &callee,
                        &"takes",
                        &typed.with_article(),
                        &format!(" for `{}`", slot.name),
                        &shown,
                        &ty.with_article(),
                    ],
                );
                continue;
            }
            let help = self.unmet_bound_help(&ty);
            if let Some(diag) = self.report(
                call.span,
                msgs::check::MISSING_IMPL_DOES_NOT_IMPLEMENT_REQUIRED,
                &[&ty, trait_name, &callee],
            ) {
                diag.help = help;
            }
        }
    }
}
