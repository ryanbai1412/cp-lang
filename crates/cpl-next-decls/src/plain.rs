//! Which types are copied and which are moved: a property of the type and its declarations.
use crate::decls::Decls;
use crate::ty::*;

/// R1: no part of the value lives on the heap, so using it is a copy.
pub fn plain(d: &Decls, t: &Ty, depth: usize) -> bool {
    plain_in(d, t, depth, &[])
}
/// Whether `t` is copied, not moved, where the type parameters `doms` are domains (R04.2).
pub fn plain_in(d: &Decls, t: &Ty, depth: usize, doms: &[usize]) -> bool {
    match t {
        Ty::Param(i) => doms.contains(i),
        Ty::Int
        | Ty::Range(..)
        | Ty::RangeOf(_)
        | Ty::RangeTo(..)
        | Ty::Bool
        | Ty::Char
        | Ty::U64
        | Ty::I128
        | Ty::F64
        | Ty::Unit
        | Ty::Dom(_)
        | Ty::Fn(..)
        | Ty::Clo(_) => true,
        Ty::Struct(id, targs, sets) if depth < 8 => {
            let s = Subst { tys: targs, sets };
            d.s(*id)
                .fields
                .iter()
                .all(|(_, ft)| plain_in(d, &s.ty(ft), depth + 1, doms))
        }
        _ => false,
    }
}
