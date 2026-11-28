//! Declarations that need themselves: an alias defined through its own name, a struct stored
//! inside itself.
use crate::decls::{AliasId, Named};
use crate::sym::Syms;
use crate::ty::*;
use crate::views::{Items, TypeTables};
use cpl_next_syntax::ast::{GenericArg, Type};

/// The types a written type is built from, read through the typed view.
pub(crate) fn parts(t: &Type) -> Vec<Type> {
    let args = |a: Option<cpl_next_syntax::ast::GenericArgs>| -> Vec<Type> {
        (a.into_iter().flat_map(|a| a.args().collect::<Vec<_>>()))
            .filter_map(|a| match a {
                GenericArg::Type(t) => Some(t),
                GenericArg::Param(_) => None,
            })
            .collect()
    };
    match t {
        Type::Path(p) => args(p.generic_args()),
        Type::Optional(o) => o.inner().into_iter().collect(),
        Type::Array(a) => {
            let dims = a
                .dims()
                .into_iter()
                .flat_map(|d| d.args().collect::<Vec<_>>());
            (dims.filter_map(|a| match a {
                GenericArg::Type(t) => Some(t),
                GenericArg::Param(_) => None,
            }))
            .chain(a.elem())
            .collect()
        }
        Type::Tuple(ts) => ts.elems().into_iter().map(|(_, t)| t).collect(),
        Type::Record(r) => r.fields().filter_map(|f| f.ty()).collect(),
        Type::Fn(f) => f
            .param()
            .into_iter()
            .chain(f.ret().and_then(|r| r.ty()))
            .collect(),
        Type::Alphabet(a) => a.base().into_iter().collect(),
        Type::DomainBinder(_)
        | Type::Range(_)
        | Type::Literal(_)
        | Type::Infer(_)
        | Type::TypeOf(_) => Vec::new(),
    }
}

/// `std.Pair` in `std.Pair<A, B>`: the path a written type names, as one name.
pub(crate) fn path_text(p: &cpl_next_syntax::ast::PathType) -> String {
    let segs: Vec<String> = p.segments().map(|t| t.text().to_string()).collect();
    segs.join(".")
}

/// Whether alias `root`'s type needs `root` itself, through its own name or other aliases (E0323).
pub fn alias_cycle(d: &dyn TypeTables, syms: &Syms, root: AliasId) -> bool {
    fn needs(
        d: &dyn TypeTables,
        syms: &Syms,
        at: AliasId,
        t: &Type,
        root: AliasId,
        seen: &mut Vec<AliasId>,
    ) -> bool {
        if let Type::Path(p) = t {
            let named = syms
                .get(&path_text(p))
                .and_then(|s| d.alias(at).names.get(&s).copied());
            if let Some(Named::Alias(b)) = named {
                if b == root {
                    return true;
                }
                if !seen.contains(&b) {
                    seen.push(b);
                    if needs(d, syms, b, &d.alias(b).ty, root, seen) {
                        return true;
                    }
                }
            }
        }
        parts(t).iter().any(|t| needs(d, syms, at, t, root, seen))
    }
    needs(d, syms, root, &d.alias(root).ty, root, &mut vec![root])
}

/// The first field of struct `root` that stores a `root` inline, however deep (R03.2.9).
/// Storage (`Raw`), references and function values end the walk: they hold their contents elsewhere.
pub fn field_holding_itself(d: &dyn Items, root: StructId) -> Option<usize> {
    fn holds(d: &dyn Items, t: &Ty, root: StructId, stack: &mut Vec<StructId>) -> bool {
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
