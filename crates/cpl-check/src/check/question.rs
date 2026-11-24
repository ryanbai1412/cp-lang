//! `?` and its extent (R02.4.7), and `!` on a whole array or list (R02.4.8).

use super::*;
use cpl_diag::msgs;
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
                let help = format!(
                    "to leave on `None`, bind it: `let v = {} else: return None` (R02.4.7)",
                    self.quote(inner.span)
                );
                if let Some(diag) = self.report(
                    expr.span,
                    msgs::check::DROPPED_QUESTION_THE_RESULT_OF_IS.at(&[&self.quote(expr.span)]),
                ) {
                    diag.help = Some(help);
                }
            }
        }
        if let (true, StmtKind::Fn(f)) = (top_level, &stmt.kind) {
            self.overload_clash(f);
        }
        let outer = std::mem::replace(&mut self.rhs_extent, stmt.rhs_extent());
        let ty = self.stmt_body(stmt, top_level);
        self.rhs_extent = outer;
        ty
    }

    /// `xs!` on `[D]?T` or `List<?T>` checks every cell: `[D]T`, `List<T>`.
    pub(super) fn unwrapped_cells(ty: Ty) -> Ty {
        match ty {
            Ty::Array { index, elem } => match *elem {
                Ty::Optional(inner) => Ty::Array { index, elem: inner },
                _ => Ty::Unknown,
            },
            Ty::Named { name, mut args } if name == "List" && args.len() == 1 => match args.pop() {
                Some(Ty::Optional(inner)) => Ty::Named {
                    name,
                    args: vec![*inner],
                },
                _ => Ty::Unknown,
            },
            _ => Ty::Unknown,
        }
    }
}
