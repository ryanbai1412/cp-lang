//! Where code puts values: the places it writes and the structs it builds.

use crate::walk;
use cpl_ir::{Arg, Block, Expr, ExprKind, Mutation, Place, Stmt, Ty};

/// How a write changes its place.
#[derive(Clone, Copy)]
pub(crate) enum Write<'a> {
    /// `place = v` or `place op= v`.
    Store,
    /// A push, pop, insert or other mutation in place.
    Mutate(&'a Mutation),
    /// Sorted, or viewed `mut` (a slice or a line of it).
    InPlace,
    /// Passed `mut` to a call.
    Mut,
}

/// Every place `block` writes, at any depth, and how: each target a write
/// could change.
pub(crate) fn each_write(block: &Block, f: &mut dyn FnMut(Write, &Place)) {
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Assign(target, ..) = stmt {
            f(Write::Store, target);
        }
    });
    walk::deep_exprs(block, &mut |e| match &e.kind {
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            for arg in args {
                if let Arg::Mut(target) = arg {
                    f(Write::Mut, target);
                }
            }
        }
        ExprKind::Mutate(how, target, _) => f(Write::Mutate(how), target),
        ExprKind::Sort(target, _, _)
        | ExprKind::MutSlice(target, ..)
        | ExprKind::MutLine(target, ..) => f(Write::InPlace, target),
        _ => {}
    });
}

/// [`each_write`], whatever the write.
pub(crate) fn each_written(block: &Block, f: &mut dyn FnMut(&Place)) {
    each_write(block, &mut |_, target| f(target));
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
