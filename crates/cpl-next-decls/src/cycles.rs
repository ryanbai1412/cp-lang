//! Declarations that need themselves: an alias defined through its own name, a struct stored
//! inside itself.
use crate::decls::{AliasId, Named};
use crate::ty::*;
use crate::views::Signatures;
use cpl_syntax::ast::{self, TypeKind};

/// Whether alias `root`'s type needs `root` itself, through its own name or other aliases (E0323).
pub fn alias_cycle(d: &dyn Signatures, root: AliasId) -> bool {
    fn needs(
        d: &dyn Signatures,
        at: AliasId,
        t: &ast::Type,
        root: AliasId,
        seen: &mut Vec<AliasId>,
    ) -> bool {
        let inner: Vec<&ast::Type> = match &t.kind {
            TypeKind::Named { name, args } => {
                if let Some(Named::Alias(b)) = d.alias(at).names.get(&name.text) {
                    if *b == root {
                        return true;
                    }
                    if !seen.contains(b) {
                        seen.push(*b);
                        if needs(d, *b, &d.alias(*b).ty, root, seen) {
                            return true;
                        }
                    }
                }
                args.iter().collect()
            }
            TypeKind::Array { index, elem } => index.iter().chain([&**elem]).collect(),
            TypeKind::OwnDomain { elem: t, .. }
            | TypeKind::Optional(t)
            | TypeKind::Mut(t)
            | TypeKind::Yields(t) => vec![&**t],
            TypeKind::Tuple(ts) => ts.iter().collect(),
            TypeKind::Record(fs) | TypeKind::Row(fs) => fs.iter().map(|(_, t)| t).collect(),
            TypeKind::Fn { params, ret } => params.iter().chain([&**ret]).collect(),
            TypeKind::RowWithout { prefix, base, .. } => {
                prefix.iter().map(|(_, t)| t).chain([&**base]).collect()
            }
            _ => Vec::new(),
        };
        inner.into_iter().any(|t| needs(d, at, t, root, seen))
    }
    needs(d, root, &d.alias(root).ty, root, &mut vec![root])
}

/// The first field of struct `root` that stores a `root` inline, however deep (R03.2.9).
/// Storage (`Raw`), references and function values end the walk: they hold their contents elsewhere.
pub fn field_holding_itself(d: &dyn Signatures, root: StructId) -> Option<usize> {
    fn holds(d: &dyn Signatures, t: &Ty, root: StructId, stack: &mut Vec<StructId>) -> bool {
        let Ty::Struct(id, tys, sets) = t else {
            return false;
        };
        if *id == root {
            return true;
        }
        if stack.contains(id) {
            return false;
        }
        stack.push(*id);
        let sub = Subst { tys, sets };
        let found = d
            .s(*id)
            .fields
            .iter()
            .any(|(_, f)| holds(d, &sub.ty(f), root, stack));
        stack.pop();
        found
    }
    d.s(root)
        .fields
        .iter()
        .position(|(_, f)| holds(d, f, root, &mut Vec::new()))
}
