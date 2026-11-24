//! A trait's sequence roles, read off its declaration: the parameter a method
//! yields (`fn iter(self) -> yields T`) is its element, and the first of a
//! yielded pair whose second is a parameter (`fn items(self) -> yields (D, T)`)
//! its key. A super's roles carry through the args it is written with:
//! `trait Indexed<D, T>: Keyed<D, T>` keys by `D`.

use super::*;

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) struct SeqRoles {
    pub key: Option<usize>,
    pub elem: Option<usize>,
}

impl<'a> Lowerer<'a> {
    pub(super) fn seq_roles(&self, name: &str) -> Option<SeqRoles> {
        // The core `IndexedMut` has no std declaration to read.
        if name == "IndexedMut" {
            return Some(SeqRoles {
                key: Some(0),
                elem: Some(1),
            });
        }
        self.roles_of(name, &mut Vec::new())
    }

    fn roles_of(&self, name: &str, seen: &mut Vec<String>) -> Option<SeqRoles> {
        if seen.iter().any(|s| s == name) {
            return None;
        }
        seen.push(name.to_string());
        let tid = match self.get_sym(name) {
            Some(Sym::Trait(tid)) => tid,
            _ => self.traits.iter().position(|t| t.name == name)?,
        };
        let t = &self.traits[tid];
        let param = |ty: &ast::Type| match &ty.kind {
            ast::TypeKind::Named { name, args } if args.is_empty() => {
                t.generics.iter().position(|g| *g == name.text)
            }
            _ => None,
        };
        let mut roles = SeqRoles::default();
        for f in &t.methods {
            let Some(ast::TypeKind::Yields(y)) = f.ret.as_ref().map(|r| &r.kind) else {
                continue;
            };
            match &y.kind {
                ast::TypeKind::Tuple(pair) if pair.len() == 2 => {
                    if let (Some(k), Some(e)) = (param(&pair[0]), param(&pair[1])) {
                        roles.key = roles.key.or(Some(k));
                        roles.elem = roles.elem.or(Some(e));
                    }
                }
                _ => roles.elem = roles.elem.or(param(y)),
            }
        }
        for sup in &t.super_tys {
            let ast::TypeKind::Named { name, args } = &sup.kind else {
                continue;
            };
            let Some(theirs) = self.roles_of(&name.text, seen) else {
                continue;
            };
            let carry = |i: Option<usize>| i.and_then(|i| args.get(i)).and_then(param);
            roles.key = roles.key.or(carry(theirs.key));
            roles.elem = roles.elem.or(carry(theirs.elem));
        }
        (roles != SeqRoles::default()).then_some(roles)
    }
}
