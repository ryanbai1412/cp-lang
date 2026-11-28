use super::{FnDecl, ImplDecl, Layer, Owner, StructDecl};
use cpl_next_ir::ty::Ty;

/// The positions in `generics` of the `const` parameters a fn writes.
pub(super) fn positions(written: &[crate::sym::Sym], generics: &[crate::sym::Sym]) -> Vec<usize> {
    (written.iter())
        .filter_map(|v| generics.iter().position(|g| g == v))
        .collect()
}

/// A fn's generic is a value one when its owner's is (a method of `Mod<const P: Int>`) or when it
/// is passed where one is expected (`P` of `fn inv<P>(x: Mod<P>)` or of `impl Show for Mod<P>`),
/// as the domain kind is inferred; a type's own are written `const P: Int` (R08.2.4).
pub(super) fn infer_values(
    fns: &mut Layer<FnDecl>,
    structs: &Layer<StructDecl>,
    impls: &Layer<ImplDecl>,
    from: usize,
) {
    fn passed(structs: &Layer<StructDecl>, t: &Ty, i: usize) -> bool {
        let go = |t: &Ty| passed(structs, t, i);
        match t {
            Ty::Struct(s, ts, _) => (ts.iter().enumerate()).any(|(k, a)| {
                (*a == Ty::Param(i) && structs[s.0].value_params.contains(&k)) || go(a)
            }),
            Ty::Raw(e) | Ty::Ref(e) | Ty::Lent(e) | Ty::Mut(e) | Ty::RangeTo(e, ..) => go(e),
            Ty::Fn(ps, r, _) => ps.iter().chain([&**r]).any(go),
            _ => false,
        }
    }
    for k in from..fns.len() {
        let f = &fns[k];
        let owner: Vec<&Ty> = match f.owner {
            Some(Owner::Impl(m)) => vec![&impls[m].for_],
            _ => Vec::new(),
        };
        let of_owner = match f.owner {
            Some(Owner::Struct(s)) => structs[s.0].value_params.clone(),
            _ => Vec::new(),
        };
        let mut values: Vec<usize> = (0..f.generics.len())
            .filter(|&i| {
                f.value_params.contains(&i)
                    || of_owner.contains(&i)
                    || (f.params.iter().map(|p| &p.ty))
                        .chain([&f.ret])
                        .chain(&f.self_ty)
                        .chain(owner.iter().copied())
                        .any(|t| passed(structs, t, i))
            })
            .collect();
        values.dedup();
        fns[k].value_params = values;
    }
}
