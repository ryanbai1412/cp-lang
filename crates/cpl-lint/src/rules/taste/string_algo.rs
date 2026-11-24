//! Condition and index shapes the taste rules share.

use super::*;

/// Extracts `(lhs, op, rhs)` from a single binary comparison `lhs op rhs`.
pub(super) fn single_cmp(e: &Expr) -> Option<(&Expr, CmpOp, &Expr)> {
    if let ExprKind::Compare { first, rest } = &walk::unparen(e).kind {
        if rest.len() == 1 {
            return Some((first, rest[0].0, &rest[0].1));
        }
    }
    None
}

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

pub(super) fn cond_expr(cond: &Cond) -> Option<&Expr> {
    match cond {
        Cond::Expr(e) => Some(e),
        _ => None,
    }
}
