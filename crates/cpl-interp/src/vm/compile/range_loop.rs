use super::jump::JumpOp;
use super::Compiler;
use crate::vm::ir::*;
use cpl_diag::Span;
use cpl_ir::{Block, LoopId, Slot};

impl<'p, 'f> Compiler<'p, 'f> {
    /// `for r_var in r_lo..r_hi` (`..=` when `closed`), the bounds already
    /// in registers. `positions_of`: the loop walks every position of that
    /// domain from 0, so checks of them can be dropped. `resumable`: a
    /// checkpoint can resume the loop from its registers.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_up_loop(
        &mut self,
        id: LoopId,
        slot: Slot,
        r_var: u16,
        tag_v: Option<u16>,
        r_lo: u16,
        r_hi: u16,
        closed: bool,
        positions_of: Option<u16>,
        resumable: bool,
        body: &'p Block,
        lo_span: Span,
        hi_span: Span,
    ) {
        let skip = if closed {
            JumpOp::JmpGtI { a: r_lo, b: r_hi }
        } else {
            JumpOp::JmpGeI { a: r_lo, b: r_hi }
        };
        let j_skip = self.emit_jump(skip, lo_span);
        if r_var != r_lo {
            self.emit(
                Op::MoveI {
                    dst: r_var,
                    src: r_lo,
                },
                lo_span,
            );
        }
        let body_pc = self.ops.len() as u32;
        self.emit_loop_tag(tag_v, r_var, lo_span);
        let marks = (self.dom_clears, self.assigned_locals.len());
        let reg = resumable.then_some(LoopReg::Range { id, r_var, r_hi });
        let lt = self.loop_body(id, reg, &body.stmts);
        let step_pc = self.ops.len() as u32;
        if let Some(d) = positions_of {
            self.drop_checks_of_positions(r_var, d, slot, body_pc, marks);
        }
        self.patch_all(lt.continues, step_pc);
        let back = self.ops.len();
        self.note_back_edge(back, reg);
        if closed {
            self.emit(
                Op::ForIncClosed {
                    r_i: r_var,
                    r_hi,
                    target: body_pc,
                },
                hi_span,
            );
        } else {
            self.emit(
                Op::ForInc {
                    r_i: r_var,
                    r_hi,
                    target: body_pc,
                },
                hi_span,
            );
        }
        let end_pc = self.ops.len() as u32;
        self.patch(j_skip, end_pc);
        self.patch_all(lt.breaks, end_pc);
    }

    /// An enum loop variable (`tag_v`) is counted by its tag in `r_var`
    /// and set from it as each round starts.
    pub(super) fn emit_loop_tag(&mut self, tag_v: Option<u16>, r_var: u16, span: Span) {
        if let Some(dst_v) = tag_v {
            self.emit(
                Op::EnumOfTagI {
                    dst_v,
                    src_i: r_var,
                },
                span,
            );
        }
    }

    /// `for r_var in (r_lo..r_hi).rev` (`..=` when `closed`), the bounds
    /// already in registers.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_down_loop(
        &mut self,
        id: LoopId,
        r_var: u16,
        tag_v: Option<u16>,
        r_lo: u16,
        r_hi: u16,
        closed: bool,
        body: &'p Block,
        lo_span: Span,
        hi_span: Span,
    ) {
        let skip = if closed {
            JumpOp::JmpGtI { a: r_lo, b: r_hi }
        } else {
            JumpOp::JmpGeI { a: r_lo, b: r_hi }
        };
        let j_skip = self.emit_jump(skip, lo_span);
        if closed {
            if r_var != r_hi {
                self.emit(
                    Op::MoveI {
                        dst: r_var,
                        src: r_hi,
                    },
                    hi_span,
                );
            }
        } else {
            self.emit(
                Op::AddImmI {
                    dst: r_var,
                    a: r_hi,
                    imm: -1,
                },
                hi_span,
            );
        }
        let body_pc = self.ops.len() as u32;
        self.emit_loop_tag(tag_v, r_var, hi_span);
        let lt = self.loop_body(id, None, &body.stmts);
        let step_pc = self.ops.len() as u32;
        self.patch_all(lt.continues, step_pc);
        self.emit(
            Op::ForDec {
                r_i: r_var,
                r_lo,
                target: body_pc,
            },
            lo_span,
        );
        let end_pc = self.ops.len() as u32;
        self.patch(j_skip, end_pc);
        self.patch_all(lt.breaks, end_pc);
    }
}
