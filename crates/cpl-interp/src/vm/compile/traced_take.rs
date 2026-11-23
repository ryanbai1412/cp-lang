use super::Compiler;
use crate::vm::helpers::quiet_place;
use crate::vm::ir::*;
use cpl_diag::Span;
use cpl_ir::{Expr, ExprKind, Mutation, Place, Stmt};

impl<'p> Compiler<'p, '_> {
    /// A `take` of a place: it leaves the place's default behind, so a
    /// traced chunk runs it and reports the place itself.
    pub(super) fn traced_take(&self, e: &'p Expr) -> bool {
        let ExprKind::Mutate(Mutation::Take, place, args) = &e.kind else {
            return false;
        };
        args.is_empty() && quiet_place(place, cpl_ir::walk::root(place)) && !self.yields_into(place)
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

    /// `return e` where `e` only writes places it names (a `pop`, a
    /// `take`): the VM runs it and reports each written place before
    /// returning, as the tree-walker does while it evaluates `e`.
    pub(super) fn returns_after_writes(&self, value: &'p Expr) -> bool {
        self.feed_acc.is_none() && self.inline_ret.is_none() && self.only_writes_places(value)
    }

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

    pub(super) fn compile_traced_writing_return(&mut self, value: &'p Expr) {
        let fallbacks = self.fallback_exprs.len();
        let scalar = ScalarKind::of_ty(&value.ty, self.program).is_some();
        let r = if scalar {
            self.compile_expr_i(value)
        } else {
            self.compile_expr_v(value)
        };
        self.trace_writes_of(value, fallbacks);
        let op = if scalar {
            Op::RetI { src: r }
        } else {
            Op::RetV { src: r }
        };
        self.emit(op, value.span);
    }

    /// `let x = e` where `e` only writes places it names, into a register
    /// local: the writes are reported before `x` takes the value.
    pub(super) fn writing_let(&self, slot: usize, init: &'p Expr) -> bool {
        matches!(
            self.slot_loc(slot),
            SlotLoc::I(..) | SlotLoc::OptI { .. } | SlotLoc::V(_)
        ) && self.only_writes_places(init)
    }

    pub(super) fn compile_traced_writing_let(&mut self, slot: usize, init: &'p Expr) {
        let fallbacks = self.fallback_exprs.len();
        let span = init.span;
        match self.slot_loc(slot) {
            SlotLoc::I(dst, _) => {
                let src = self.compile_expr_i(init);
                self.trace_writes_of(init, fallbacks);
                if src != dst {
                    self.emit(Op::MoveI { dst, src }, span);
                }
                self.in_dom.retain(|f| f.0 != dst);
            }
            SlotLoc::OptI { val, some, .. } => {
                let (rv, rs) = self.compile_expr_opt_i(init);
                self.trace_writes_of(init, fallbacks);
                for (dst, src) in [(val, rv), (some, rs)] {
                    if src != dst {
                        self.emit(Op::MoveI { dst, src }, span);
                    }
                }
            }
            SlotLoc::V(dst) => {
                let src = self.compile_expr_v(init);
                self.trace_writes_of(init, fallbacks);
                if src != dst {
                    self.emit(Op::MoveV { dst, src }, span);
                }
            }
            SlotLoc::SmallTuple { .. } => unreachable!("not a writing let"),
        }
        self.emit_trace(TracePoint::Let(slot, init), span);
    }

    /// Reports the places `e` wrote, unless the tree-walker ran part of it
    /// (any fallback since `fallbacks`) and reported them itself.
    pub(super) fn trace_writes_of(&mut self, e: &'p Expr, fallbacks: usize) {
        if self.fallback_exprs.len() != fallbacks {
            return;
        }
        if let Some(writes) = super::stmt::cond_writes(e) {
            for (place, span) in writes {
                self.emit_trace(TracePoint::Mutated(place, span), span);
            }
        }
    }
}
