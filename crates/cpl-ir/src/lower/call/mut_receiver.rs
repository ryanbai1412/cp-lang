//! The receiver of a method that changes it in place.

use super::*;

impl<'a> Lowerer<'a> {
    /// Where `method`'s `mut self` lives. A temporary has nowhere to keep
    /// the change, so the error names the method, not an assignment.
    pub(in crate::lower) fn mut_receiver(&mut self, recv: &ast::Expr, method: &str) -> Place {
        if !is_place(recv) && self.builtin_at(recv).is_none() {
            let span = self.span(recv.span);
            self.error(
                codes::ASSIGN_TO_LET,
                recv.span,
                format!(
                    "`{method}` changes its receiver in place, so it needs a mutable place; bind this value to a `var` first"
                ),
            );
            return Place {
                kind: PlaceKind::Local(0),
                ty: Ty::Error,
                span,
            };
        }
        self.place(recv)
    }

    /// The place a `mut` argument names; a receiver goes through
    /// [`Self::mut_receiver`].
    pub(in crate::lower) fn mut_arg(
        &mut self,
        value: &ast::Expr,
        param: &str,
        method: &str,
    ) -> Place {
        if param == "self" {
            self.mut_receiver(value, method)
        } else {
            self.place(value)
        }
    }
}
