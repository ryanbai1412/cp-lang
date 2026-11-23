//! Branches decided at compile time: an `if` whose condition folds to a
//! constant (an `O: Trait` test, `same(op, +)`) keeps only the branch it
//! takes, so the C++ carries neither the other branch nor the fns only it
//! calls. A build without checks drops `assert`s here for the same reason.

use cpl_ir::{BinOp, Block, Expr, ExprKind, Program, Stmt, UnOp};

/// `asserts` is whether the build checks them; when it does not, they go
/// too, so a fn only an `assert` calls is not emitted.
pub(crate) fn prune(program: &mut Program, asserts: bool) {
    for f in &mut program.fns {
        block(&mut f.body, asserts);
    }
    for c in &mut program.closures {
        block(&mut c.body, asserts);
    }
    block(&mut program.main, asserts);
    block(&mut program.setup, asserts);
}

/// The value of a condition made only of `true`, `false`, `not`, `and` and
/// `or`; a side that is not known is evaluated only when the known one lets
/// it be skipped.
pub(crate) fn known(e: &Expr) -> Option<bool> {
    match &e.kind {
        ExprKind::Bool(b) => Some(*b),
        ExprKind::Unary(UnOp::Not, x) => known(x).map(|b| !b),
        ExprKind::Binary(BinOp::And, a, b) => match known(a)? {
            false => Some(false),
            true => known(b),
        },
        ExprKind::Binary(BinOp::Or, a, b) => match known(a)? {
            true => Some(true),
            false => known(b),
        },
        _ => None,
    }
}

fn block(b: &mut Block, asserts: bool) {
    if !asserts {
        b.stmts.retain(|s| !matches!(s, Stmt::Assert { .. }));
    }
    for stmt in &mut b.stmts {
        match stmt {
            Stmt::If(cond, then, otherwise) => {
                if let Some(taken) = known(cond) {
                    cond.kind = ExprKind::Bool(taken);
                    let dead = if taken { &mut *otherwise } else { &mut *then };
                    dead.stmts.clear();
                }
                block(then, asserts);
                block(otherwise, asserts);
            }
            Stmt::While(_, _, body) | Stmt::For(_, _, _, body) => block(body, asserts),
            _ => {}
        }
    }
}
