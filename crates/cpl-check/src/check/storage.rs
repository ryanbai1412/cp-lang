//! Writes need storage behind the place (R07.2.5).

use super::lends::show;
use super::*;
use cpl_syntax::ast::{Expr, ExprKind};

impl Checker<'_> {
    /// The type of an assignment's target, which must have storage.
    pub(super) fn assign_target(&mut self, target: &Expr) -> Ty {
        let ty = self.infer(target);
        self.assign_into_range(target);
        ty
    }

    /// `r[i] = x` where `r` is a `Range`: positions with no data to write.
    fn assign_into_range(&mut self, target: &Expr) {
        let ExprKind::Index { base, .. } = &target.kind else {
            return;
        };
        let base: &Expr = base;
        let base_ty = self.quietly(|c| c.infer(base));
        if !matches!(&base_ty, Ty::Named { name, .. } if name == "Range") {
            return;
        }
        let base = show(base);
        let message = format!("`{base}` is a `Range`, which has no storage to write");
        if let Some(diag) = self.error(codes::ASSIGN_INTO_RANGE, target.span, message) {
            diag.help = Some(format!("copy it with `List({base})` first"));
        }
    }
}
