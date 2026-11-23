//! Small readers of statement and expression shapes shared by the loop rules.

use crate::walk::unparen;
use cpl_syntax::ast::*;

pub(crate) fn name(e: &Expr) -> Option<&str> {
    match &unparen(e).kind {
        ExprKind::Name(n) => Some(n),
        _ => None,
    }
}

pub(crate) fn int(e: &Expr) -> Option<i128> {
    match unparen(e).kind {
        ExprKind::Int(v) => Some(v),
        _ => None,
    }
}

pub(crate) fn single(block: &Block) -> Option<&Stmt> {
    match block.stmts.as_slice() {
        [s] => Some(s),
        _ => None,
    }
}

/// `x op y` as one comparison.
pub(crate) fn compare(e: &Expr) -> Option<(&Expr, CmpOp, &Expr)> {
    let ExprKind::Compare { first, rest } = &unparen(e).kind else {
        return None;
    };
    match rest.as_slice() {
        [(op, rhs)] => Some((first, *op, rhs)),
        _ => None,
    }
}

/// `var += 1`.
pub(crate) fn is_step(stmt: &Stmt, var: &str) -> bool {
    matches!(&stmt.kind, StmtKind::Assign { target, op: AssignOp::Binary(BinOp::Add), value }
        if name(target) == Some(var) && int(value) == Some(1))
}
