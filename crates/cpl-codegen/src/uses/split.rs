//! What code computes and where its values land: every expression in the
//! order it runs, the values a `do` block gives, and each `let`, loop
//! binding, store and return.

use crate::walk;
use cpl_ir::{AssignOp, Block, Expr, ExprKind, Iter, Place, Slot, Stmt};

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
    /// The binding of a `for` loop, one item per pass, and the sequence it
    /// walks (none for a range).
    For(Slot, Option<&'a Expr>),
    /// `place = v` or `place op= v`: the place, how it combines, the value.
    Store(&'a Place, &'a AssignOp, &'a Expr),
    /// `return v`.
    Return(&'a Expr),
    /// `give v`: the value of the `do` block it ends.
    Give(&'a Expr),
}

/// Every `let`, loop binding, store, return and give in `block`, at any depth (a
/// `do` block's too), in statement order.
pub(crate) fn each_sink<'a>(block: &'a Block, f: &mut dyn FnMut(Sink<'a>)) {
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::Let(slot, init) => f(Sink::Let(*slot, init.as_ref())),
        Stmt::For(_, slot, iter, _) => f(Sink::For(
            *slot,
            match iter {
                Iter::Seq(seq, _) => Some(seq),
                _ => None,
            },
        )),
        Stmt::Assign(place, op, value) => f(Sink::Store(place, op, value)),
        Stmt::Return(Some(e)) => f(Sink::Return(e)),
        Stmt::Give(e) => f(Sink::Give(e)),
        _ => {}
    });
}

/// Every statement in `block`, at any depth (a `do` block's too), in order:
/// `Some((Some(slot), init))` for a `let` with a value, `Some((None, value))`
/// for a store `p = value`, `None` for any other. A run of `Some` is a run
/// of `let`s and stores with nothing between them.
pub(crate) fn each_let_run<'a>(
    block: &'a Block,
    f: &mut dyn FnMut(Option<(Option<Slot>, &'a Expr)>),
) {
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::Let(slot, Some(init)) => f(Some((Some(*slot), init))),
        Stmt::Assign(_, AssignOp::Set, value) => f(Some((None, value))),
        _ => f(None),
    });
}

/// The value of every `give` in `block`, at any depth: what each `Do` block
/// hands out.
pub(crate) fn each_give<'a>(block: &'a Block, f: &mut dyn FnMut(&'a Expr)) {
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Give(e) = stmt {
            f(e);
        }
    });
}
