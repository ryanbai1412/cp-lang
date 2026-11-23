//! Compile-time integers, one evaluation shared by the checker and the
//! lowerer, so both agree on what is constant (R11.1.1).

use crate::ast::{BinOp, Expr, ExprKind, UnOp};

/// The value of `e` when it is an integer known at compile time; `name`
/// gives the value of a name, or `None` when it is not a constant.
pub fn eval(e: &Expr, name: &dyn Fn(&str) -> Option<i64>) -> Option<i64> {
    match &e.kind {
        ExprKind::Int(v) => i64::try_from(*v).ok(),
        ExprKind::Paren(inner) => eval(inner, name),
        ExprKind::Unary {
            op: UnOp::Pos,
            expr,
        } => eval(expr, name),
        ExprKind::Unary {
            op: UnOp::Neg,
            expr,
        } => eval(expr, name)?.checked_neg(),
        ExprKind::Binary { op, lhs, rhs, .. } => {
            let l = eval(lhs, name)?;
            let r = eval(rhs, name)?;
            match op {
                BinOp::Add => l.checked_add(r),
                BinOp::Sub => l.checked_sub(r),
                BinOp::Mul => l.checked_mul(r),
                BinOp::Div if r != 0 => l.checked_div_euclid(r),
                BinOp::Rem if r != 0 => l.checked_rem_euclid(r),
                BinOp::Pow if r >= 0 => l.checked_pow(u32::try_from(r).ok()?),
                BinOp::Shl if (0..63).contains(&r) => l.checked_mul(1 << r),
                BinOp::Shr if (0..64).contains(&r) => Some(l >> r),
                BinOp::BitAnd => Some(l & r),
                BinOp::BitOr => Some(l | r),
                BinOp::BitXor => Some(l ^ r),
                _ => None,
            }
        }
        ExprKind::Name(n) => name(n),
        _ => None,
    }
}

/// Whether a `domain` defined as `def` has compile-time bounds
/// (`0..5`, `[N]`), so it is defined before the statements above it.
pub fn constant_domain(def: &Expr, name: &dyn Fn(&str) -> Option<i64>) -> bool {
    match &def.kind {
        ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            step: None,
            ..
        } => eval(lo, name).is_some() && eval(hi, name).is_some(),
        ExprKind::List(items) => matches!(items.as_slice(), [n] if eval(n, name).is_some()),
        _ => false,
    }
}
