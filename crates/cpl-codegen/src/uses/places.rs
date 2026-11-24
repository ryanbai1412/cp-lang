//! Where code puts values: the places it writes and the structs it builds.

use crate::walk;
use cpl_ir::{Arg, Block, Expr, ExprKind, Place, Stmt, Ty};

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

/// Every struct literal in `block`, at any depth: the struct and its
/// fields. A struct value is only ever made by one (every other value of
/// it is a copy of one).
pub(crate) fn each_built(block: &Block, f: &mut dyn FnMut(usize, &[Expr])) {
    walk::deep_exprs(block, &mut |e| {
        if let (ExprKind::Tuple(fields), Ty::Struct(s)) = (&e.kind, &e.ty) {
            f(*s, fields);
        }
    });
}
