//! What code computes and where its values land: every expression in the
//! order it runs, the values a `do` block gives, and each `let`, loop
//! binding, store and return.

use crate::walk;
use cpl_ir::{Block, Expr, ExprKind, Place, Slot, Stmt};

/// Every expression in `block`, at any depth (a `do` block's too), each
/// after the expressions inside it: the order code computes them.
pub(crate) fn each_value<'a>(block: &'a Block, f: &mut dyn FnMut(&'a Expr)) {
    for stmt in &block.stmts {
        walk::stmt_exprs(stmt, &mut |e| value(e, f));
        walk::stmt_blocks(stmt, &mut |inner| each_value(inner, f));
    }
}

fn value<'a>(e: &'a Expr, f: &mut dyn FnMut(&'a Expr)) {
    if let ExprKind::Do(block) = &e.kind {
        each_value(block, f);
    }
    walk::children(e, &mut |child| value(child, f));
    f(e);
}

/// The values a `do` block gives: its own `give`s, at any depth of its
/// statements, not those of a `do` nested inside.
pub(crate) fn gives(block: &Block) -> Vec<&Expr> {
    fn go<'a>(block: &'a Block, out: &mut Vec<&'a Expr>) {
        for stmt in &block.stmts {
            if let Stmt::Give(e) = stmt {
                out.push(e);
            }
            walk::stmt_blocks(stmt, &mut |inner| go(inner, out));
        }
    }
    let mut out = Vec::new();
    go(block, &mut out);
    out
}

/// Where a statement puts a value.
#[derive(Clone, Copy)]
pub(crate) enum Sink<'a> {
    /// `let slot = init`, or a `var` with no initializer.
    Let(Slot, Option<&'a Expr>),
    /// The binding of a `for` loop, one item per pass.
    For(Slot),
    /// `place = v` or `place op= v`.
    Store(&'a Place, &'a Expr),
    /// `return v`.
    Return(&'a Expr),
}

/// Every `let`, loop binding, store and return in `block`, at any depth (a
/// `do` block's too), in statement order.
pub(crate) fn each_sink<'a>(block: &'a Block, f: &mut dyn FnMut(Sink<'a>)) {
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::Let(slot, init) => f(Sink::Let(*slot, init.as_ref())),
        Stmt::For(_, slot, ..) => f(Sink::For(*slot)),
        Stmt::Assign(place, _, value) => f(Sink::Store(place, value)),
        Stmt::Return(Some(e)) => f(Sink::Return(e)),
        _ => {}
    });
}

/// Every statement in `block`, at any depth (a `do` block's too), in order:
/// `Some((slot, init))` for a `let` with a value, `None` for any other. A
/// run of `Some` is a run of `let`s with nothing between them.
pub(crate) fn each_let_run<'a>(block: &'a Block, f: &mut dyn FnMut(Option<(Slot, &'a Expr)>)) {
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::Let(slot, Some(init)) => f(Some((*slot, init))),
        _ => f(None),
    });
}
