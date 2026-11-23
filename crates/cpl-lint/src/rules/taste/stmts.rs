//! Readers of statement and expression shapes the taste rules share.

use crate::walk::{self, Visitor};
use cpl_syntax::ast::*;

pub(super) fn var_decl(stmt: &Stmt) -> Option<(&str, &Expr)> {
    match &stmt.kind {
        StmtKind::Let {
            mutable: true,
            pat:
                Pattern {
                    kind: PatternKind::Name(name),
                    ..
                },
            init: Some(init),
            otherwise: None,
            guard: None,
            ..
        } => Some((name, init)),
        _ => None,
    }
}

/// `stmts` assign no place but `name`: calls (`print`, `seen.add(v)`) may
/// run, but no other variable or table moves.
pub(super) fn only_sets(stmts: &[Stmt], name: &str) -> bool {
    stmts.iter().all(|s| match &s.kind {
        StmtKind::Assign { target, .. } => {
            matches!(&target.kind, ExprKind::Name(n) if n == name)
        }
        StmtKind::Break(None) | StmtKind::Continue(None) => true,
        StmtKind::Let { .. } => true,
        StmtKind::Expr(Expr {
            kind: ExprKind::If {
                branches,
                otherwise,
            },
            ..
        }) => {
            branches.iter().all(|(_, b)| only_sets(&b.stmts, name))
                && otherwise.as_ref().is_none_or(|b| only_sets(&b.stmts, name))
        }
        StmtKind::Expr(_) => true,
        _ => false,
    })
}

/// Some block under `stmts` sets `name` and then breaks.
pub(super) fn breaks_after_setting(stmts: &[Stmt], name: &str) -> bool {
    let sets = |s: &Stmt| {
        matches!(&s.kind, StmtKind::Assign { target, op: AssignOp::Set, .. }
            if matches!(&target.kind, ExprKind::Name(n) if n == name))
    };
    stmts
        .windows(2)
        .any(|w| sets(&w[0]) && matches!(w[1].kind, StmtKind::Break(None)))
        || stmts.iter().any(|s| match &s.kind {
            StmtKind::Expr(Expr {
                kind:
                    ExprKind::If {
                        branches,
                        otherwise,
                    },
                ..
            }) => {
                branches
                    .iter()
                    .any(|(_, b)| breaks_after_setting(&b.stmts, name))
                    || otherwise
                        .as_ref()
                        .is_some_and(|b| breaks_after_setting(&b.stmts, name))
            }
            StmtKind::Unchecked(b) => breaks_after_setting(&b.stmts, name),
            _ => false,
        })
}

/// `xs.push(e)`: `e`.
pub(super) fn pushed_onto<'e>(e: &'e Expr, name: &str) -> Option<&'e Expr> {
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    let ExprKind::Field { base, name: method } = &callee.kind else {
        return None;
    };
    match (&base.kind, &args[..]) {
        (ExprKind::Name(n), [arg]) if n == name && method.text == "push" && arg.name.is_none() => {
            Some(&arg.value)
        }
        _ => None,
    }
}

/// `stmt` holds a `?`.
pub(super) fn has_try(stmt: &Stmt) -> bool {
    struct Tries(bool);
    impl Visitor for Tries {
        fn expr(&mut self, expr: &Expr) {
            self.0 |= matches!(expr.kind, ExprKind::Try(_));
        }
    }
    let mut tries = Tries(false);
    walk::walk_stmt(&mut tries, stmt);
    tries.0
}

/// Whether `e` writes a place: `x min= y`, `mut x`, or a `mut` argument.
pub(super) fn writes_place(e: &Expr) -> bool {
    struct Writes(bool);
    impl Visitor for Writes {
        fn expr(&mut self, expr: &Expr) {
            self.0 |= match &expr.kind {
                ExprKind::Keep { .. } | ExprKind::Mut(_) => true,
                ExprKind::Call { args, .. } => args.iter().any(|a| a.mutable),
                _ => false,
            };
        }
    }
    let mut writes = Writes(false);
    walk::walk_expr(&mut writes, e);
    writes.0
}

/// A test passed to a fold: a one-parameter lambda or a name.
pub(super) fn is_test(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Lambda { params, .. } => params.len() == 1,
        ExprKind::Name(_) | ExprKind::Field { .. } => true,
        _ => false,
    }
}

/// Whether `.m()` binds to all of `e` without parentheses.
pub(super) fn postfix(e: &Expr) -> bool {
    matches!(
        e.kind,
        ExprKind::Name(_)
            | ExprKind::Field { .. }
            | ExprKind::Call { .. }
            | ExprKind::Index { .. }
            | ExprKind::Paren(_)
            | ExprKind::List(_)
            | ExprKind::Tuple(_)
            | ExprKind::Str(_)
    )
}
