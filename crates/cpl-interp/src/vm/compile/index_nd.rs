//! Reads and stores of an array of 3 or more dims, whose indices sit in a
//! run of registers.

use super::Compiler;
use crate::vm::ir::*;
use cpl_diag::Span;
use cpl_ir::{Expr, Ty};

impl<'p, 'f> Compiler<'p, 'f> {
    /// True when `indices` name one cell of a `base` with 3 or more dims.
    pub(super) fn indexes_nd_cell(base: &Ty, indices: &[Expr]) -> bool {
        matches!(base, Ty::Array { dims, .. } if dims.len() > 2 && dims.len() == indices.len())
    }

    /// Evaluates `indices` in order into a run of registers: its first and
    /// its length.
    pub(super) fn compile_index_run(&mut self, indices: &'p [Expr]) -> (u16, u16) {
        let regs: Vec<u16> = indices.iter().map(|i| self.compile_expr_i(i)).collect();
        let first = self.next_i;
        for (src, index) in regs.into_iter().zip(indices) {
            let dst = self.alloc_i();
            self.emit(Op::MoveI { dst, src }, index.span);
        }
        (first, indices.len() as u16)
    }

    /// `base[indices]` as a value, for a `base` that `indexes_nd_cell`.
    pub(super) fn compile_index_nd_v(
        &mut self,
        base: &'p Expr,
        indices: &'p [Expr],
        span: Span,
    ) -> u16 {
        let arr_v = self.compile_expr_v(base);
        let (idx, n) = self.compile_index_run(indices);
        let dst = self.alloc_v();
        self.emit(Op::LoadArrNV { dst, arr_v, idx, n }, span);
        if arr_v >= self.num_local_v {
            self.emit(Op::ClearV { reg: arr_v }, span);
        }
        dst
    }
}
