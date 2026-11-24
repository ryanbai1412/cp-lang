//! Where code puts values: the places it writes.

use crate::walk;
use cpl_ir::{Arg, Block, ExprKind, Place, Stmt};

/// Every place `block` stores to, sets in place or passes `mut`, at any
/// depth: each target a write could change.
pub(crate) fn each_written(block: &Block, f: &mut dyn FnMut(&Place)) {
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Assign(target, ..) = stmt {
            f(target);
        }
    });
    walk::deep_exprs(block, &mut |e| match &e.kind {
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            for arg in args {
                if let Arg::Mut(target) = arg {
                    f(target);
                }
            }
        }
        ExprKind::Mutate(_, target, _) | ExprKind::Sort(target, _, _) => f(target),
        _ => {}
    });
}
