//! Free extension fns, `fn f(self: S, ..)`, met as `x.f(..)`.

use super::*;
use cpl_syntax::ast;

impl<'a> Checker<'a> {
    /// E0203 for `a.push(4)` on a `[3]Int` when every `push` in reach is a
    /// free fn whose `self` names another type (std's
    /// `push(mut self: Counter<K, Int>, ..)`): the receiver has no such
    /// method, which is the root cause, not its mismatch with that `self`.
    pub(super) fn no_extension_takes(&mut self, base_ty: &Ty, name: &ast::Name) -> bool {
        let owner = match base_ty {
            Ty::Str => "Str",
            Ty::Array { .. } => "[D]T",
            Ty::Named { name, .. } if name == "List" || name == "Range" => name.as_str(),
            Ty::Range { .. } => "Range",
            _ => return false,
        };
        if crate::builtins::builtin_member(owner, &name.text).is_some() {
            return false;
        }
        let defs: Vec<_> = [
            self.global(&name.text),
            self.global(&crate::prelude::std_method(&name.text)),
        ]
        .into_iter()
        .flatten()
        .collect();
        let mut takes = Vec::new();
        for def in &defs {
            let Def::Fn {
                ty: Ty::Fn { params, .. },
                ..
            } = &**def
            else {
                return false;
            };
            let Some(first) = params.first().filter(|p| p.name == "self") else {
                return false;
            };
            let want = first.ty.clone();
            let other = match (&want, base_ty) {
                (Ty::Named { name: w, .. }, Ty::Named { name: b, .. }) => w != b,
                (Ty::Named { .. }, _) => true,
                _ => false,
            };
            if !other || self.fits(base_ty, &want) {
                return false;
            }
            takes.push(want);
        }
        let Some(want) = takes.first() else {
            return false;
        };
        let widened = base_ty.widen().with_article();
        let (a, ty) = widened.split_once(' ').unwrap_or(("a", &widened));
        let message = format!("{a} `{ty}` has no method `{}`", name.text);
        let note = format!("the `{}` in reach takes a `{want}` as `self`", name.text);
        if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
            diag.help = Some(note);
        }
        true
    }
}
