//! Condition and index shapes the taste rules share.

use super::*;

/// Matches `var - 1`.
pub(super) fn is_var_minus_one(e: &Expr, var: &str) -> bool {
    matches!(
        &walk::unparen(e).kind,
        ExprKind::Binary { op: BinOp::Sub, lhs, rhs, .. }
            if shapes::name(lhs) == Some(var) && is_int_lit(rhs, 1)
    )
}

/// Matches `var + 1` or `1 + var`.
pub(super) fn is_var_plus_one(e: &Expr, var: &str) -> bool {
    matches!(
        &walk::unparen(e).kind,
        ExprKind::Binary { op: BinOp::Add, lhs, rhs, .. }
            if (shapes::name(lhs) == Some(var) && is_int_lit(rhs, 1))
                || (shapes::name(rhs) == Some(var) && is_int_lit(lhs, 1))
    )
}
