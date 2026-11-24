//! Operators on user types (W9): the impl method `a + b` or `-a` calls when
//! `a` is a struct.

use super::*;

impl<'a> Lowerer<'a> {
    /// Looks up a binary or unary operator trait method on a concrete struct (`Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`).
    pub(in crate::lower) fn struct_op_fn(&mut self, sid: StructId, method: &str) -> Option<FnId> {
        let id = self.structs[sid].methods.get(method).copied()?;
        self.queue_fn(id);
        Some(id)
    }

    /// Looks up a binary operator trait method matching `(method, rhs_ty)`, falling back to homogeneous `Self`.
    pub(in crate::lower) fn struct_op_fn_for(
        &mut self,
        sid: StructId,
        method: &str,
        rhs_ty: &Ty,
    ) -> Option<(FnId, Ty)> {
        if let Some((_, want, id)) = self.structs[sid]
            .op_methods
            .iter()
            .find(|(m, want, _)| m == method && want == rhs_ty)
            .cloned()
        {
            self.queue_fn(id);
            return Some((id, want));
        }
        let id = self.structs[sid].methods.get(method).copied()?;
        self.queue_fn(id);
        Some((id, Ty::Struct(sid)))
    }
}
