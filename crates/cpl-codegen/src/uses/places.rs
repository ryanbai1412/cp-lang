//! Where code puts values: the places it writes, the structs it builds,
//! the fns it passes them to and the closures it makes.

use crate::walk;
use cpl_ir::{
    Arg, AssignOp, Block, ClosureId, Expr, ExprKind, FnId, Mutation, Place, Slot, Stmt, Ty,
};

/// Where a statement puts a whole value.
#[derive(Clone, Copy)]
pub(crate) enum Put<'a> {
    /// `let slot = v`.
    Let(Slot),
    /// `place = v` or `place op= v`.
    Store(&'a Place, &'a AssignOp),
}

/// Every `let` with a value and every store in `block`, at any depth and in
/// order: where it puts the value, and the value.
pub(crate) fn each_put<'p>(block: &'p Block, f: &mut dyn FnMut(Put<'p>, &'p Expr)) {
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::Let(slot, Some(v)) => f(Put::Let(*slot), v),
        Stmt::Assign(target, op, v) => f(Put::Store(target, op), v),
        _ => {}
    });
}

/// How a write changes its place.
#[derive(Clone, Copy)]
pub(crate) enum Write<'a> {
    /// `place = v` or `place op= v`.
    Store,
    /// A push, pop, insert or other mutation in place.
    Mutate(&'a Mutation),
    /// Sorted in place.
    Sort,
    /// Viewed `mut`: a slice or a line of it.
    InPlace,
    /// Passed `mut` to a call.
    Mut,
}

/// Every place `block` writes, at any depth, and how: each target a write
/// could change.
pub(crate) fn each_write<'p>(block: &'p Block, f: &mut dyn FnMut(Write<'p>, &'p Place)) {
    for stmt in &block.stmts {
        each_write_in(stmt, f);
    }
}

/// [`each_write`] for one statement and everything under it.
pub(crate) fn each_write_in<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(Write<'p>, &'p Place)) {
    stores_in(stmt, &mut |target| f(Write::Store, target));
    stmt_deep_exprs(stmt, &mut |e| match &e.kind {
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            for arg in args {
                if let Arg::Mut(target) = arg {
                    f(Write::Mut, target);
                }
            }
        }
        ExprKind::Mutate(how, target, _) => f(Write::Mutate(how), target),
        ExprKind::Sort(target, _, _) => f(Write::Sort, target),
        ExprKind::MutSlice(target, ..) | ExprKind::MutLine(target, ..) => f(Write::InPlace, target),
        _ => {}
    });
}

fn stores_in<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(&'p Place)) {
    if let Stmt::Assign(target, ..) = stmt {
        f(target);
    }
    walk::stmt_exprs(stmt, &mut |e| {
        walk::deep_expr(e, &mut |inner| {
            if let ExprKind::Do(block) = &inner.kind {
                block.stmts.iter().for_each(|s| stores_in(s, f));
            }
        })
    });
    walk::stmt_blocks(stmt, &mut |inner| {
        inner.stmts.iter().for_each(|s| stores_in(s, f));
    });
}

fn stmt_deep_exprs<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(&'p Expr)) {
    walk::stmt_exprs(stmt, &mut |e| walk::deep_expr(e, f));
    walk::stmt_blocks(stmt, &mut |inner| walk::deep_exprs(inner, f));
}

/// Every tuple or struct literal in `stmt`, at any depth: its items.
pub(crate) fn each_tuple_in<'p>(stmt: &'p Stmt, f: &mut dyn FnMut(&'p [Expr])) {
    stmt_deep_exprs(stmt, &mut |e| {
        if let ExprKind::Tuple(items) = &e.kind {
            f(items);
        }
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

/// Every call by name in `block`, at any depth: the fn and its arguments.
pub(crate) fn each_call(block: &Block, f: &mut dyn FnMut(FnId, &[Arg])) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Call(callee, args) = &e.kind {
            f(*callee, args);
        }
    });
}

/// Every closure `block` makes, at any depth.
pub(crate) fn each_closure(block: &Block, f: &mut dyn FnMut(ClosureId)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Closure(cid) = e.kind {
            f(cid);
        }
    });
}

/// Every call of a fn value in `block`, at any depth: the callee and its
/// arguments.
pub(crate) fn each_value_call(block: &Block, f: &mut dyn FnMut(&Expr, &[Arg])) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::CallExpr(callee, args) = &e.kind {
            f(callee, args);
        }
    });
}

/// Every optional `block` tests (`x != None`) or reads through (`x!`), at
/// any depth: the optional.
pub(crate) fn each_opened(block: &Block, f: &mut dyn FnMut(&Expr)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::IsSome(x) | ExprKind::Unwrap(x, false) = &e.kind {
            f(x);
        }
    });
}

/// Every expression inside `e`, `e` itself included, at any depth.
pub(crate) fn each_part<'p>(e: &'p Expr, f: &mut dyn FnMut(&'p Expr)) {
    walk::deep_expr(e, f);
}

/// Every expression statement in `block`, at any depth: the expression.
pub(crate) fn each_stmt_expr<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Expr)) {
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Expr(e) = stmt {
            f(e);
        }
    });
}

/// Every local `block` reads, at any depth.
pub(crate) fn each_local(block: &Block, f: &mut dyn FnMut(Slot)) {
    walk::deep_exprs(block, &mut |e| {
        if let ExprKind::Local(s) = e.kind {
            f(s);
        }
    });
}

/// Every `return` in `block`, at any depth: its value, if any.
pub(crate) fn each_return(block: &Block, f: &mut dyn FnMut(Option<&Expr>)) {
    walk::deep_stmts(block, &mut |s| {
        if let Stmt::Return(value) = s {
            f(value.as_ref());
        }
    });
}
