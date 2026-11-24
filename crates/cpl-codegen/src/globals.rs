//! The globals a block writes or reads in place, which main can't keep
//! as its own locals.

use std::collections::BTreeSet;

use cpl_ir::{Block, ExprKind, Stmt};

pub(crate) fn touched(block: &Block, out: &mut BTreeSet<usize>) {
    crate::walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Assign(place, _, _) = stmt {
            if let (true, id) = crate::walk::root(place) {
                out.insert(id);
            }
        }
    });
    crate::walk::deep_exprs(block, &mut |e| match &e.kind {
        ExprKind::Global(id) => {
            out.insert(*id);
        }
        ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, _, _) => {
            if let (true, id) = crate::walk::root(place) {
                out.insert(id);
            }
        }
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            for arg in args {
                if let cpl_ir::Arg::Mut(place) = arg {
                    if let (true, id) = crate::walk::root(place) {
                        out.insert(id);
                    }
                }
            }
        }
        _ => {}
    });
}
