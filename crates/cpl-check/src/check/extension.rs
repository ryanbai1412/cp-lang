//! Free extension fns, `fn f(self: S, ..)`, met as `x.f(..)`.

use super::*;
use cpl_syntax::ast;

impl<'a> Checker<'a> {
    /// E0203 for `a.push(4)` on a `[3]Int` when every `push` in reach is a
    /// free fn whose `self` names another type (std's
    /// `push(mut self: Counter<K, Int>, ..)`): the receiver has no such
    /// method, which is the root cause, not its mismatch with that `self`.
    pub(super) fn no_extension_takes(&mut self, base_ty: &Ty, name: &ast::Name) -> bool {
        let Some(want) = self.no_extension_fits(base_ty, &name.text) else {
            return false;
        };
        let message = Self::no_method(base_ty, name);
        let note = format!("the `{}` in reach takes a `{want}` as `self`", name.text);
        if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
            diag.help = Some(note);
        }
        true
    }

    /// "a `[3]Int` has no method `push`".
    pub(super) fn no_method(base_ty: &Ty, name: &ast::Name) -> String {
        let widened = base_ty.widen().with_article();
        let (a, ty) = widened.split_once(' ').unwrap_or(("a", &widened));
        format!("{a} `{ty}` has no method `{}`", name.text)
    }

    /// The did-you-mean for `name` on `base_ty`: a member it has, never a
    /// free fn whose `self` is another type.
    pub(super) fn member_typo(&mut self, base_ty: &Ty, name: &str) -> Option<String> {
        let owner = receiver_owner(base_ty);
        super::spec_member_typo(base_ty, name, &mut |s| {
            self.no_extension_fits(base_ty, s).is_none()
                && !owner.is_some_and(|o| foreign_self(o, s))
        })
    }

    /// The `self` type of the `name` in reach when every `name` is a free fn
    /// whose `self` names a type other than `base_ty`; a did-you-mean never
    /// offers such a name (errors/R7).
    pub(super) fn no_extension_fits(&mut self, base_ty: &Ty, name: &str) -> Option<Ty> {
        let owner = receiver_owner(base_ty)?;
        if crate::builtins::builtin_member(owner, name).is_some() {
            return None;
        }
        let defs: Vec<_> = (self.method_candidates(name).into_iter())
            .map(|(_, def)| def)
            .collect();
        let mut takes = Vec::new();
        for def in &defs {
            let Def::Fn {
                ty: Ty::Fn { params, .. },
                ..
            } = &**def
            else {
                return None;
            };
            let first = params.first().filter(|p| p.name == "self")?;
            let want = first.ty.clone();
            let other = match (&want, base_ty) {
                (Ty::Named { name: w, .. }, Ty::Named { name: b, .. }) => w != b,
                (Ty::Named { .. }, _) => true,
                _ => false,
            };
            if !other || self.fits(base_ty, &want) {
                return None;
            }
            takes.push(want);
        }
        takes.into_iter().next()
    }
}

/// The built-in receiver `base_ty` is, as `builtin_member` names it.
fn receiver_owner(base_ty: &Ty) -> Option<&str> {
    Some(match base_ty {
        Ty::Str => "Str",
        Ty::Array { .. } => "[D]T",
        Ty::Named { name, .. } if name == "List" || name == "Range" => name.as_str(),
        Ty::Range { .. } => "Range",
        _ => return None,
    })
}

/// Whether every std `fn name(self: T, ..)` names a `T` that is not the
/// built-in receiver `owner`, even before `name` is in reach (`halve` takes
/// a `Range`, so a `List` is never offered it).
fn foreign_self(owner: &str, name: &str) -> bool {
    if crate::builtins::builtin_member(owner, name).is_some() {
        return false;
    }
    let takes = |head: &str| match owner {
        "Str" => head == "Str",
        "Range" => head == "Range",
        _ => head == "List" || head == "[",
    };
    crate::prelude::std_self_heads()
        .get(name)
        .is_some_and(|heads| {
            heads
                .iter()
                .all(|h| h.as_deref().is_some_and(|h| !takes(h)))
        })
}
