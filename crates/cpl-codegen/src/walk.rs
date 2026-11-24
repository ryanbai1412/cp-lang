//! Walking the IR: see [`cpl_ir::walk`].

pub(crate) use cpl_ir::walk::*;

use cpl_ir::{Block, ExprKind, SortBy, Stmt};

/// Calls `f` on every slot `block` binds, at any depth: a `let`, a loop
/// variable, a sort key's element.
pub(crate) fn binders(block: &Block, f: &mut dyn FnMut(usize)) {
    deep_stmts(block, &mut |s| match s {
        Stmt::Let(slot, _) | Stmt::For(_, slot, ..) => f(*slot),
        _ => {}
    });
    deep_exprs(block, &mut |e| {
        if let ExprKind::Sort(_, SortBy::Key(slot, _), _) = &e.kind {
            f(*slot);
        }
    });
}
