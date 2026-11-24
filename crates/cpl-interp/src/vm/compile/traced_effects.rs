//! The one hook through which a traced chunk reports a write an
//! expression does in place (`xs.pop()`, `take(x)`): right after it runs,
//! as the tree-walker reports it while it evaluates the expression.

use super::Compiler;
use crate::vm::ir::*;
use cpl_ir::{Expr, ExprKind, Place, Stmt};

impl<'p> Compiler<'p, '_> {
    /// The place `e` writes in place, in a traced chunk, unless `e` is
    /// already being compiled by [`Self::reporting_write`].
    pub(super) fn reported_write(&self, e: &'p Expr) -> Option<&'p Place> {
        let ExprKind::Mutate(_, place, _) = &e.kind else {
            return None;
        };
        let outer = self.reporting.is_some_and(|r| std::ptr::eq(r, e));
        (self.traced && !outer).then_some(&**place)
    }

    /// Compiles `e` by `compile`, then reports `place`, unless the
    /// tree-walker ran `e` and reported it.
    pub(super) fn reporting_write<R>(
        &mut self,
        e: &'p Expr,
        place: &'p Place,
        compile: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let outer = self.reporting.replace(e);
        let out = compile(self);
        self.reporting = outer;
        let walked = (self.fallback_exprs.last()).is_some_and(|f| std::ptr::eq(*f, e));
        if !walked {
            self.emit_trace(TracePoint::Mutated(place, e.span), e.span);
        }
        out
    }

    /// Compiles `stmt` whose value is `value` as the untraced chunk would.
    /// `value`'s own write is reported where it runs when it compiles as an
    /// expression, else after the statement, by a path that takes the write
    /// and the store as one op.
    pub(super) fn compile_reporting_stmt(&mut self, stmt: &'p Stmt, value: &'p Expr) {
        let points = self.trace_points.len();
        self.compile_untraced_stmt(stmt);
        let Some(place) = self.reported_write(value) else {
            return;
        };
        let reported = (self.trace_points[points..].iter())
            .any(|p| matches!(p, TracePoint::Mutated(_, span) if *span == value.span));
        let walked = self.fell_back(stmt)
            || (self.fallback_exprs.last()).is_some_and(|f| std::ptr::eq(*f, value));
        if !reported && !walked {
            self.emit_trace(TracePoint::Mutated(place, value.span), value.span);
        }
    }

    /// A `sort` of a global shows at the next step, as in the tree-walker.
    /// The statements of a `do` block, which belong to the expression
    /// around them: a loop whose iterable holds one starts no round in it.
    pub(super) fn compile_do_stmts(&mut self, stmts: &'p [Stmt]) {
        let round = self.traced_loop_var.take();
        self.compile_stmts(stmts);
        self.traced_loop_var = round;
    }

    pub(super) fn trace_sorted(&mut self, stmt: &'p Stmt, place: &'p Place) {
        if !self.fell_back(stmt) && cpl_ir::walk::root(place).0 {
            self.emit_trace(TracePoint::Sorted, place.span);
        }
    }
}
