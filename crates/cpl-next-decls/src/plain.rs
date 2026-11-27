//! Which types are copied and which are moved: a property of the type and its declarations.
use crate::decls::Decls;
use crate::ty::*;
use std::collections::BTreeSet;

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
        | Ty::Bool
        | Ty::Char
        | Ty::U64
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

/// The parameters a call to `f` lends to its result, receiver first, from its signature: the
/// heap ones it declares `lends` (R05.6.1), else each whose type is a type parameter the result
/// type holds, and, for a result whose type is a type parameter, each that holds it: the result
/// may be a cell of it (R05.5.3).
pub fn lends(d: &Decls, f: FnId) -> BTreeSet<usize> {
    use crate::decls::SelfMode;
    use cpl_next_ir::ty::params_in;
    let fd = d.f(f);
    let mut held = BTreeSet::new();
    params_in(&fd.ret, &mut held);
    let recv = (fd.self_mode != SelfMode::None).then_some(fd.self_ty.as_ref());
    let tys: Vec<&Ty> = (recv.flatten().into_iter())
        .chain(fd.params.iter().map(|p| &p.ty))
        .collect();
    (tys.iter().enumerate())
        .filter(|(i, t)| match &fd.lends {
            Some(ls) => ls.contains(i) && !plain(d, t, 0),
            None => match (t, &fd.ret) {
                (Ty::Param(k), _) => held.contains(k),
                (_, Ty::Param(k)) => {
                    let mut ps = BTreeSet::new();
                    params_in(t, &mut ps);
                    ps.contains(k)
                }
                _ => false,
            },
        })
        .map(|(i, _)| i)
        .collect()
}
