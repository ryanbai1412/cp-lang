//! `reverse_index` (L0528): `[xs[xs.len - 1 - i] for i in 0..xs.len]`
//! builds `xs.reversed()` by hand. The list is labelled so the fact table
//! keeps the finding only on a `List` (`typed.rs`).

use crate::walk::Visitor;
use crate::{File, Finding, REVERSE_INDEX};
use cpl_diag::{Diagnostic, Fix};
use cpl_syntax::ast::{BinOp, Expr, ExprKind};

use super::shapes::binary;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Reverse<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Reverse<'_> {
        fn expr(&mut self, expr: &Expr) {
            let Some(list) = self.reversed(expr) else {
                return;
            };
            let span = expr.span.shift(self.base);
            let call = format!("{}.reversed()", list.span.shift(self.base).text(self.src));
            let mut diag = Diagnostic::warning(
                REVERSE_INDEX.code,
                span,
                format!("this copies the list backwards by index: it is `{call}`"),
            )
            .with_label(list.span.shift(self.base), "the list")
            .with_help(format!(
                "write `{call}`; to walk backwards without a copy, use `.rev`"
            ));
            diag.fixes.push(Fix {
                title: format!("use `{call}`"),
                edits: vec![(span, call)],
            });
            self.out.push(Finding {
                lint: &REVERSE_INDEX,
                diag,
            });
        }
    }
    impl Reverse<'_> {
        fn reversed<'e>(&self, expr: &'e Expr) -> Option<&'e Expr> {
            let ExprKind::Comprehension { body, clauses } = &expr.kind else {
                return None;
            };
            let [clause] = &clauses[..] else {
                return None;
            };
            let ExprKind::Range {
                lo,
                hi: Some(hi),
                inclusive: false,
                step: None,
            } = &clause.iter.kind
            else {
                return None;
            };
            if !clause.conds.is_empty()
                || lo
                    .as_ref()
                    .is_some_and(|lo| !matches!(lo.kind, ExprKind::Int(0)))
            {
                return None;
            }
            let ExprKind::Index {
                base: list,
                indices,
            } = &body.kind
            else {
                return None;
            };
            let [index] = &indices[..] else {
                return None;
            };
            let (BinOp::Sub, last, i) = binary(index)? else {
                return None;
            };
            let (BinOp::Sub, len, one) = binary(last)? else {
                return None;
            };
            let text = |e: &Expr| e.span.shift(self.base).text(self.src);
            let place = text(list);
            let whole = format!("{place}.len");
            let path = place
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '.');
            let var = clause.pat.span.shift(self.base).text(self.src);
            (path
                && matches!(one.kind, ExprKind::Int(1))
                && text(len) == whole
                && text(hi) == whole
                && text(i) == var)
                .then_some(&**list)
        }
    }
    let mut v = Reverse {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}
