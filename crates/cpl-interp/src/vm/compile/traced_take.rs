use super::Compiler;
use crate::vm::helpers::quiet_place;
use cpl_ir::{Expr, ExprKind};

impl<'p> Compiler<'p, '_> {
    /// Whether `e`'s only effects are writes to places it names, each
    /// reported by a `Trace` after the VM runs `e`.
    pub(super) fn only_writes_places(&self, value: &'p Expr) -> bool {
        let Some(writes) = super::stmt::cond_writes(value) else {
            return false;
        };
        let mut loud = false;
        cpl_ir::walk::deep_expr(value, &mut |e| {
            loud |= matches!(
                e.kind,
                ExprKind::Call(..)
                    | ExprKind::CallExpr(..)
                    | ExprKind::Print(..)
                    | ExprKind::Debug(..)
                    | ExprKind::Sort(..)
                    | ExprKind::MutSlice(..)
            )
        });
        !loud
            && !writes.is_empty()
            && (writes.iter())
                .all(|(p, _)| quiet_place(p, cpl_ir::walk::root(p)) && !self.yields_into(p))
    }
}
