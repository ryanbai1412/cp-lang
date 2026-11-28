//! A struct's field types under its type arguments: what `Ty::holds` reads of a declaration.
use crate::ty::*;
use crate::views::Items;

/// The field types of a struct type, for `Ty::holds` and `Ty::copied`; none for other types.
pub fn fields(d: &dyn Items) -> impl Fn(&Ty) -> Vec<Ty> + '_ {
    move |t| match t {
        Ty::Struct(id, targs, sets) => {
            let s = Subst { tys: targs, sets };
            (d.s(*id).fields.iter()).map(|(_, ft)| s.ty(ft)).collect()
        }
        _ => Vec::new(),
    }
}
