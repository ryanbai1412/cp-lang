//! A run at a judge build's [`Checks`]: where the level keeps no check, the
//! operation does what that build does (R09.3) instead of trapping, as
//! `cpl_ir::checks` says for the code generator too.

use crate::{trap, Res};
use cpl_diag::Span;
use cpl_ir::checks::{Check, Checks};
use cpl_ir::BinOp;
use std::cell::Cell;

thread_local! {
    static CHECKS: Cell<Checks> = const { Cell::new(Checks::LOCAL) };
}

/// The run's checks, from [`Set::new`] until it is dropped.
pub(crate) struct Set(Checks);

impl Set {
    pub(crate) fn new(checks: Checks) -> Set {
        Set(CHECKS.with(|c| c.replace(checks)))
    }
}

impl Drop for Set {
    fn drop(&mut self) {
        CHECKS.with(|c| c.set(self.0));
    }
}

pub(crate) fn keeps(check: Check) -> bool {
    CHECKS.with(|c| c.get().keeps(check))
}

/// An `Int` result that does not fit: the trap, or the `wrapped` bits
/// release keeps (R09.3.4).
#[cold]
pub(crate) fn overflow<T>(span: Span, message: String, wrapped: T) -> Res<T> {
    if keeps(Check::Overflow) {
        trap(span, message)
    } else {
        Ok(wrapped)
    }
}

/// A zero divisor: the trap, or release's fallback, 0 (R09.3.6).
#[cold]
pub(crate) fn div_zero<T: Default>(span: Span, message: String) -> Res<T> {
    if keeps(Check::Div) {
        trap(span, message)
    } else {
        Ok(T::default())
    }
}

/// `a op b` on 64 bits, keeping what fits: release's overflow.
pub(crate) fn wrapping(op: BinOp, a: i64, b: i64) -> i64 {
    match op {
        BinOp::Add => a.wrapping_add(b),
        BinOp::Sub => a.wrapping_sub(b),
        BinOp::Mul => a.wrapping_mul(b),
        BinOp::Div => a.wrapping_div_euclid(b),
        BinOp::Rem => a.wrapping_rem_euclid(b),
        BinOp::Pow => a.wrapping_pow(b as u32),
        BinOp::Shl => a.wrapping_shl(b as u32),
        _ => 0,
    }
}
