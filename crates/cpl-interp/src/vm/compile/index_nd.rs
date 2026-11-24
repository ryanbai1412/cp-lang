//! Reads and stores of an array of 3 or more dims, whose indices sit in a
//! run of registers.

use super::Compiler;
use crate::vm::ir::*;
use cpl_diag::Span;
use cpl_ir::{Expr, ExprKind, Ty};

impl<'p, 'f> Compiler<'p, 'f> {
    /// The path of `base[index]` for `LoadNestedArrI`: `base` indexes arrays
    /// or lists nested any depth, each by one index, from a global or a boxed
    /// local. Evaluates the indices, outermost first.
    pub(super) fn nested_read_path(&mut self, base: &'p Expr, index: &'p Expr) -> Option<u16> {
        let mut indices = vec![index];
        let mut cur = base;
        let root = loop {
            match &cur.kind {
                ExprKind::Index(b, ix)
                    if ix.len() == 1 && matches!(b.ty, Ty::Array { .. } | Ty::List(_)) =>
                {
                    indices.push(&ix[0]);
                    cur = b;
                }
                ExprKind::Global(gid) => break PlaceRoot::Global(*gid as u16),
                ExprKind::Local(slot) => match self.slot_loc(*slot) {
                    SlotLoc::V(v) => break PlaceRoot::LocalV(v),
                    _ => return None,
                },
                _ => return None,
            }
        };
        let mut steps = Vec::with_capacity(indices.len());
        for k in (0..indices.len()).rev() {
            steps.push(PlaceStep::Index1(self.compile_expr_i(indices[k])));
        }
        let idx = u16::try_from(self.places.len()).ok()?;
        self.places.push(PlacePath {
            root,
            steps,
            keys: Vec::new(),
        });
        Some(idx)
    }

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
        self.free_temp_v(arr_v, span);
        dst
    }
}
