//! `compound_self_ref` (L0210): `x = x + e` is `x += e`, one place written
//! once instead of two copies that must agree.

use crate::walk::{walk_expr, Visitor};
use crate::{File, Finding, COMPOUND_SELF_REF};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::{AssignOp, BinOp, Expr, ExprKind, Stmt, StmtKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Assigns<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Assigns<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            let StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            } = &stmt.kind
            else {
                return;
            };
            let ExprKind::Binary { op, lhs, rhs, .. } = &value.kind else {
                return;
            };
            // R11: a compound assignment runs its right side before the
            // target's indices, so the rewrite is exact only when neither
            // side calls anything.
            if matches!(op, BinOp::And | BinOp::Or) || calls(target) || calls(rhs) {
                return;
            }
            let text = |e: &Expr| e.span.shift(self.base).text(self.src);
            let squash = |s: &str| s.split_whitespace().collect::<String>();
            if squash(text(lhs)) != squash(text(target)) {
                return;
            }
            let place = text(target);
            let op = format!("{}=", op.text());
            let span = Span::new(target.span.start, value.span.end).shift(self.base);
            let mut diag = Diagnostic::warning(
                COMPOUND_SELF_REF.code,
                span,
                format!("`{place}` is written in terms of itself; this is `{place} {op} ...`"),
            )
            .with_help(format!("write `{place} {op} {}`", text(rhs)));
            diag.fixes.push(Fix {
                title: format!("use `{op}`"),
                edits: vec![(
                    Span::new(target.span.end, rhs.span.start).shift(self.base),
                    format!(" {op} "),
                )],
            });
            self.out.push(Finding {
                lint: &COMPOUND_SELF_REF,
                diag,
            });
        }
    }
    let mut v = Assigns {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

/// Whether `expr` calls anything (or writes, with `min=` as a value).
fn calls(expr: &Expr) -> bool {
    struct Calls(bool);
    impl Visitor for Calls {
        fn expr(&mut self, expr: &Expr) {
            if matches!(expr.kind, ExprKind::Call { .. } | ExprKind::Keep { .. }) {
                self.0 = true;
            }
        }
    }
    let mut c = Calls(false);
    walk_expr(&mut c, expr);
    c.0
}
