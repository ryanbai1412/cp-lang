//! `neg_mod_fixup` (L0527): `(e + m) % m` is the C++ guard against a
//! negative remainder; CP Lang's `%` is Euclidean, so it is `e % m`. A
//! divisor that is not a literal is labelled so the fact table keeps the
//! finding only on a signed integer type (`typed.rs`).

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, NEG_MOD_FIXUP};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::{BinOp, Expr, ExprKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Euclid<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Euclid<'_> {
        fn expr(&mut self, expr: &Expr) {
            let ExprKind::Binary {
                op: BinOp::Rem,
                lhs,
                rhs: m,
                ..
            } = &expr.kind
            else {
                return;
            };
            let ExprKind::Paren(sum) = &lhs.kind else {
                return;
            };
            let ExprKind::Binary {
                op: BinOp::Add,
                lhs: e,
                rhs: added,
                ..
            } = &sum.kind
            else {
                return;
            };
            let text = |e: &Expr| e.span.shift(self.base).text(self.src);
            let divisor = text(m);
            if text(added) != divisor || divisor.contains('(') {
                return;
            }
            let label = match m.kind {
                ExprKind::Int(_) => false,
                ExprKind::Name(_) | ExprKind::Field { .. } => true,
                _ => return,
            };
            let inner = unparen(e);
            let edit = match &inner.kind {
                ExprKind::Binary {
                    op: BinOp::Rem,
                    rhs,
                    ..
                } if text(rhs) == divisor => (expr.span, text(inner).to_string()),
                _ => (Span::new(e.span.end, added.span.end), String::new()),
            };
            let span = expr.span.shift(self.base);
            let mut fixed = span.text(self.src).to_string();
            let at = (edit.0.start - expr.span.start) as usize;
            fixed.replace_range(at..at + edit.0.len() as usize, &edit.1);
            let mut diag = Diagnostic::warning(
                NEG_MOD_FIXUP.code,
                span,
                format!("`+ {divisor}` guards against a negative remainder, which `%` never gives"),
            )
            .with_help(format!(
                "write `{fixed}`; `%` is Euclidean, so the result is already in `0..{divisor}`"
            ));
            if label {
                diag = diag.with_label(m.span.shift(self.base), "the divisor");
            }
            diag.fixes.push(Fix {
                title: format!("use `{fixed}`"),
                edits: vec![(edit.0.shift(self.base), edit.1)],
            });
            self.out.push(Finding {
                lint: &NEG_MOD_FIXUP,
                diag,
            });
        }
    }
    let mut v = Euclid {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}
