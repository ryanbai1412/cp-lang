//! What a written operator computes: the one map from the surface operators to the IR's
//! operations, which constant evaluation and lowering both read.

use cpl_next_ir::ir::Intr;
use cpl_next_ir::ops::{self, Operand};
use cpl_next_syntax::ast::BinExpr;
use cpl_next_syntax::ops::{BinOp, Kind, UnOp};

/// The checked operation a binary operator on scalars is; `None` for one that is not a
/// single scalar operation (`and`, `or`, `??`, `<=>`).
pub fn bin_intr(op: BinOp) -> Option<Intr> {
    Some(match op {
        BinOp::Add => Intr::Add,
        BinOp::Sub => Intr::Sub,
        BinOp::Mul => Intr::Mul,
        BinOp::Div => Intr::Div,
        BinOp::Rem => Intr::Rem,
        BinOp::BitAnd => Intr::BitAnd,
        BinOp::BitOr => Intr::BitOr,
        BinOp::BitXor => Intr::BitXor,
        BinOp::Pow => Intr::Pow,
        BinOp::Shl => Intr::Shl,
        BinOp::Shr => Intr::Shr,
        BinOp::And | BinOp::Or | BinOp::Coalesce | BinOp::Spaceship => return None,
    })
}

/// The operation a prefix operator is; `None` for `+`, which computes nothing, and `~`.
pub fn un_intr(op: UnOp) -> Option<Intr> {
    match op {
        UnOp::Neg => Some(Intr::Neg),
        UnOp::Not => Some(Intr::Not),
        UnOp::Pos | UnOp::BitNot => None,
    }
}

/// What a binary operator computes, when it computes a value rather than a comparison.
pub fn bin_op(b: &BinExpr) -> Option<BinOp> {
    match cpl_next_syntax::ops::binary(b.op()?.kind())?.kind {
        Kind::Bin(op) => Some(op),
        Kind::Cmp(_) => None,
    }
}

/// `op` on constant `Int`s, as every backend runs it: an overflow or a fault folds to nothing,
/// so the end is not a constant and the run reports it (R09.3).
pub fn fold(op: Intr, xs: &[i64]) -> Option<i64> {
    match *xs {
        [a] => {
            let w = ops::unary(op, ops::sig(op, Operand::Int), [a as u64, 0]).ok()?;
            Some(w[0] as i64)
        }
        [a, b] => ops::int(op, a, b).ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_fold_as_the_run_does() {
        let at = |op| bin_intr(op).unwrap();
        assert_eq!(fold(at(BinOp::Pow), &[10, 9]), Some(1_000_000_000));
        assert_eq!(fold(at(BinOp::Shl), &[1, 4]), Some(16));
        assert_eq!(fold(at(BinOp::Shr), &[-17, 2]), Some(-5));
        assert_eq!(fold(un_intr(UnOp::Neg).unwrap(), &[5]), Some(-5));
        // Overflow is the run-time trap, so it folds to nothing.
        assert_eq!(fold(at(BinOp::Pow), &[2, 63]), None);
        assert_eq!(fold(at(BinOp::Shl), &[1, 63]), None);
        assert_eq!(fold(at(BinOp::Mul), &[i64::MAX, 2]), None);
        assert_eq!(fold(at(BinOp::Div), &[1, 0]), None);
        assert_eq!(bin_intr(BinOp::Coalesce), None);
    }
}
