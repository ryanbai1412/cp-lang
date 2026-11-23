//! `ceil_div_formula` (L0526): `(a + b - 1) / b` rounds up by hand;
//! `ceil_div(a, b)` names it. A divisor that is not a positive literal is
//! labelled so the fact table can keep the finding only where its range
//! type is positive (`typed.rs`).

use crate::walk::Visitor;
use crate::{File, Finding, CEIL_DIV_FORMULA};
use cpl_diag::{Diagnostic, Fix};
use cpl_syntax::ast::{BinOp, Expr, ExprKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Ceil<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Ceil<'_> {
        fn expr(&mut self, expr: &Expr) {
            let Some((a, b, positive)) = self.ceil(expr) else {
                return;
            };
            let call = format!("ceil_div({a}, {b})");
            let span = expr.span.shift(self.base);
            let mut diag = Diagnostic::warning(
                CEIL_DIV_FORMULA.code,
                span,
                format!("`{}` rounds `{a} / {b}` up by hand", span.text(self.src)),
            )
            .with_help(format!(
                "write `{call}`; it cannot overflow and is right for any sign"
            ));
            if let Some(divisor) = positive {
                diag = diag.with_label(divisor.span.shift(self.base), "the divisor");
            }
            diag.fixes.push(Fix {
                title: format!("use `{call}`"),
                edits: vec![(span, call)],
            });
            self.out.push(Finding {
                lint: &CEIL_DIV_FORMULA,
                diag,
            });
        }
    }
    impl<'e> Ceil<'_> {
        /// `a`, `b` and, unless `b` is a positive literal, the divisor
        /// whose type must prove it positive.
        fn ceil(&self, expr: &'e Expr) -> Option<(&str, &str, Option<&'e Expr>)> {
            let (BinOp::Div, num, b) = binary(expr)? else {
                return None;
            };
            let ExprKind::Paren(num) = &num.kind else {
                return None;
            };
            let (BinOp::Sub, sum, one) = binary(num)? else {
                return None;
            };
            let (BinOp::Add, x, y) = binary(sum)? else {
                return None;
            };
            if !matches!(one.kind, ExprKind::Int(1)) {
                return None;
            }
            let text = |e: &Expr| e.span.shift(self.base).text(self.src);
            let a = match text(b) {
                t if t == text(y) => x,
                t if t == text(x) => y,
                _ => return None,
            };
            if [a, b].iter().any(|e| text(e).contains('(')) {
                return None;
            }
            let positive = match b.kind {
                ExprKind::Int(v) if v > 0 => None,
                ExprKind::Name(_) | ExprKind::Field { .. } => Some(b),
                _ => return None,
            };
            Some((text(a), text(b), positive))
        }
    }
    let mut v = Ceil {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

fn binary(e: &Expr) -> Option<(BinOp, &Expr, &Expr)> {
    match &e.kind {
        ExprKind::Binary { op, lhs, rhs, .. } => Some((*op, lhs, rhs)),
        _ => None,
    }
}
