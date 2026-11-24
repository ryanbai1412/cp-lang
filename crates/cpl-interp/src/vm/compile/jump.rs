//! Forward jumps: emitted before their target is known, patched once it is.
//! A `Jump` is only made by emitting a `JumpOp`, so patching always rewrites a jump.

use super::super::ir::Op;
use super::Compiler;
use cpl_diag::Span;
use cpl_ir::LoopId;

/// A jump op without its target.
#[derive(Clone, Copy)]
pub(super) enum JumpOp {
    Jmp,
    JmpEscape {
        brk: bool,
        id: LoopId,
        innermost: bool,
    },
    JmpIfZero {
        reg: u16,
    },
    JmpIfNonZero {
        reg: u16,
    },
    JmpLtI {
        a: u16,
        b: u16,
    },
    JmpLeI {
        a: u16,
        b: u16,
    },
    JmpGtI {
        a: u16,
        b: u16,
    },
    JmpGeI {
        a: u16,
        b: u16,
    },
    JmpEqI {
        a: u16,
        b: u16,
    },
    JmpNeI {
        a: u16,
        b: u16,
    },
    ForStepEnter {
        r_lo: u16,
        r_hi: u16,
        r_step: u16,
        closed: bool,
        rev: bool,
    },
    JmpEager,
}

impl JumpOp {
    /// The op jumping to `target`.
    pub(super) fn at(self, target: u32) -> Op {
        match self {
            JumpOp::Jmp => Op::Jmp { target },
            JumpOp::JmpEscape { brk, id, innermost } => Op::JmpEscape {
                brk,
                id,
                innermost,
                target,
            },
            JumpOp::JmpIfZero { reg } => Op::JmpIfZero { reg, target },
            JumpOp::JmpIfNonZero { reg } => Op::JmpIfNonZero { reg, target },
            JumpOp::JmpLtI { a, b } => Op::JmpLtI { a, b, target },
            JumpOp::JmpLeI { a, b } => Op::JmpLeI { a, b, target },
            JumpOp::JmpGtI { a, b } => Op::JmpGtI { a, b, target },
            JumpOp::JmpGeI { a, b } => Op::JmpGeI { a, b, target },
            JumpOp::JmpEqI { a, b } => Op::JmpEqI { a, b, target },
            JumpOp::JmpNeI { a, b } => Op::JmpNeI { a, b, target },
            JumpOp::ForStepEnter {
                r_lo,
                r_hi,
                r_step,
                closed,
                rev,
            } => Op::ForStepEnter {
                r_lo,
                r_hi,
                r_step,
                closed,
                rev,
                skip: target,
            },
            JumpOp::JmpEager => Op::JmpEager { target },
        }
    }
}

/// An emitted jump still waiting for its target.
#[must_use]
pub(super) struct Jump {
    pc: u32,
    op: JumpOp,
}

impl Compiler<'_, '_> {
    pub(super) fn emit_jump(&mut self, op: JumpOp, span: Span) -> Jump {
        let pc = self.emit(op.at(0), span) as u32;
        Jump { pc, op }
    }

    pub(super) fn patch(&mut self, jump: Jump, target: u32) {
        self.ops[jump.pc as usize] = jump.op.at(target);
    }

    pub(super) fn patch_all(&mut self, jumps: Vec<Jump>, target: u32) {
        for jump in jumps {
            self.patch(jump, target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_jump_op_jumps_where_it_is_patched_to() {
        let id: LoopId = 0;
        let ops = [
            JumpOp::Jmp,
            JumpOp::JmpEscape {
                brk: true,
                id,
                innermost: false,
            },
            JumpOp::JmpIfZero { reg: 1 },
            JumpOp::JmpIfNonZero { reg: 1 },
            JumpOp::JmpLtI { a: 1, b: 2 },
            JumpOp::JmpLeI { a: 1, b: 2 },
            JumpOp::JmpGtI { a: 1, b: 2 },
            JumpOp::JmpGeI { a: 1, b: 2 },
            JumpOp::JmpEqI { a: 1, b: 2 },
            JumpOp::JmpNeI { a: 1, b: 2 },
            JumpOp::ForStepEnter {
                r_lo: 1,
                r_hi: 2,
                r_step: 3,
                closed: true,
                rev: false,
            },
            JumpOp::JmpEager,
        ];
        for op in ops {
            let target = match op.at(7) {
                Op::Jmp { target }
                | Op::JmpEscape { target, .. }
                | Op::JmpIfZero { target, .. }
                | Op::JmpIfNonZero { target, .. }
                | Op::JmpLtI { target, .. }
                | Op::JmpLeI { target, .. }
                | Op::JmpGtI { target, .. }
                | Op::JmpGeI { target, .. }
                | Op::JmpEqI { target, .. }
                | Op::JmpNeI { target, .. }
                | Op::ForStepEnter { skip: target, .. }
                | Op::JmpEager { target } => target,
                other => panic!("{other:?} is not a jump"),
            };
            assert_eq!(target, 7);
        }
    }
}
