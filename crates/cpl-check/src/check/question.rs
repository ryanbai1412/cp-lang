//! `?` and its extent (R02.4.7).

use super::*;
use cpl_syntax::ast::{ExprKind, Stmt, StmtKind};

impl Checker<'_> {
    pub(super) fn stmt(&mut self, stmt: &Stmt, top_level: bool) -> Ty {
        if top_level {
            if let StmtKind::Expr(expr) = &stmt.kind {
                self.dropped_result(expr);
            }
        }
        if let StmtKind::Expr(expr) = &stmt.kind {
            if let ExprKind::Try(inner) = &expr.kind {
                let message = format!(
                    "the result of `{}` is dropped: `?` makes its expression `None`, it never returns early",
                    self.quote(expr.span)
                );
                let help = format!(
                    "to leave on `None`, bind it: `let v = {} else: return None` (R02.4.7)",
                    self.quote(inner.span)
                );
                if let Some(diag) = self.error(codes::DROPPED_QUESTION, expr.span, message) {
                    diag.help = Some(help);
                }
            }
        }
        let outer = std::mem::replace(&mut self.rhs_extent, stmt.rhs_extent());
        let ty = self.stmt_body(stmt, top_level);
        self.rhs_extent = outer;
        ty
    }
}
