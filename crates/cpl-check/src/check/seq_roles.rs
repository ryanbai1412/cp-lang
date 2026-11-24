//! A trait's element, read off its declaration as the IR's `seq_roles` does: the
//! parameter a method yields (`fn iter(self) -> yields T`), or the first half's
//! partner of a yielded pair of parameters (`fn items(self) -> yields (D, T)`).
//! A super passes its element on through the args it is written with, so
//! `trait Rows<T>: Seq<T>` holds `T` as `Seq<T>` does.

use super::*;
use crate::ty::Ty;
use cpl_syntax::ast;

impl Checker<'_> {
    /// The args each super of a trait is written with, resolved in its body.
    pub(super) fn super_args(&mut self, supers: &[ast::Type]) -> Vec<Vec<Ty>> {
        (supers.iter())
            .filter_map(|s| match &s.kind {
                ast::TypeKind::Named { name, args } if !name.text.is_empty() => Some(
                    args.iter()
                        .map(|a| self.quietly(|c| c.resolve_type(a)))
                        .collect(),
                ),
                _ => None,
            })
            .collect()
    }

    /// Which of `trait_name`'s args is the element its sequences hold.
    pub(super) fn seq_elem_role(&mut self, trait_name: &str) -> Option<usize> {
        self.elem_role(trait_name, &mut Vec::new())
    }

    fn elem_role(&mut self, name: &str, seen: &mut Vec<String>) -> Option<usize> {
        if seen.iter().any(|s| s == name) {
            return None;
        }
        seen.push(name.to_string());
        let found = self.type_def(name);
        let Some(Def::Trait {
            generics,
            supers,
            super_args,
            methods,
            ..
        }) = found.as_deref()
        else {
            return None;
        };
        let param = |t: &Ty| match t {
            Ty::Param(g) => generics.iter().position(|p| p == g),
            _ => None,
        };
        let yielded = methods.iter().find_map(|m| match &m.ty {
            Ty::Fn { ret, .. } => match &**ret {
                Ty::Named { name, args } if name == "Gen" && args.len() == 1 => match &args[0] {
                    Ty::Tuple(pair) if pair.len() == 2 => param(&pair[0]).and(param(&pair[1])),
                    y => param(y),
                },
                _ => None,
            },
            _ => None,
        });
        yielded.or_else(|| {
            supers.iter().zip(super_args).find_map(|(sup, args)| {
                let theirs = self.elem_role(sup, seen)?;
                args.get(theirs).and_then(param)
            })
        })
    }
}
