//! A parameter typed by a trait makes its fn generic in that argument's
//! type (R08.2.1): `fn total(s: Seq<Int>)` is checked as
//! `fn total<S: Seq<Int>>(s: S)`, for a std trait and a user one alike.

use std::borrow::Cow;

use super::*;
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
