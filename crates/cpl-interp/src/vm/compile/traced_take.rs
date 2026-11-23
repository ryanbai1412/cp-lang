use super::Compiler;
use crate::vm::helpers::quiet_place;
use crate::vm::ir::*;
use cpl_diag::Span;
use cpl_ir::{Expr, ExprKind, Mutation, Place, Stmt};

impl<'p> Compiler<'p, '_> {
    /// A `take` of a scalar or optional-scalar place: it leaves the value
    /// in place, so a traced chunk runs it and reports the place itself.
    pub(super) fn traced_take(&self, e: &'p Expr) -> bool {
        let ExprKind::Mutate(Mutation::Take, place, args) = &e.kind else {
            return false;
        };
        args.is_empty()
            && (ScalarKind::of_ty(&e.ty, self.program).is_some()
                || ScalarKind::of_opt_ty(&e.ty, self.program).is_some())
            && quiet_place(place, cpl_ir::walk::root(place))
            && !self.yields_into(place)
    }

    /// `place = take(src)` in a traced chunk: `src` is reported when it is
    /// taken and `place` after the store, in the tree-walker's order.
    pub(super) fn compile_traced_take_assign(&mut self, stmt: &'p Stmt, place: &'p Place) {
        self.trace_takes = true;
        self.compile_untraced_stmt(stmt);
        self.trace_takes = false;
        if !self.fell_back(stmt) {
            let point = self.cell_point(place).unwrap_or(TracePoint::Assign(place));
            self.emit_trace(point, place.span);
        }
    }

    pub(super) fn note_take(&mut self, place: &'p Place, span: Span) {
        if self.trace_takes {
            self.emit_trace(TracePoint::Mutated(place, span), span);
        }
    }
}
