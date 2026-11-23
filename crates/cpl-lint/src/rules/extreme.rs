//! `sort_for_extreme` (L0572): `xs.sorted()[0]` or `xs.sorted().last!`
//! sorts a copy to read one element; `xs.min()!` / `xs.max()!` is one pass.
//! The list is labelled so the fact table keeps the finding only on a
//! `List` or an array (`typed.rs`).

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, SORT_FOR_EXTREME};
use cpl_diag::{Diagnostic, Fix};
use cpl_syntax::ast::{Arg, Expr, ExprKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Extremes<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Extremes<'_> {
        fn expr(&mut self, expr: &Expr) {
            let (sorted, pick) = match &expr.kind {
                ExprKind::Index { base, indices } if matches!(&indices[..], [i] if matches!(i.kind, ExprKind::Int(0))) => {
                    (&**base, "min")
                }
                ExprKind::Unwrap(inner) => match &unparen(inner).kind {
                    ExprKind::Field { base, name } if name.text == "first" => (&**base, "min"),
                    ExprKind::Field { base, name } if name.text == "last" => (&**base, "max"),
                    _ => return,
                },
                _ => return,
            };
            let Some((list, args)) = sorted_call(unparen(sorted)) else {
                return;
            };
            let at = |e: &Expr| e.span.shift(self.base).text(self.src).to_string();
            let args: Vec<String> = args
                .iter()
                .map(|a| format!("by: {}", at(&a.value)))
                .collect();
            let call = format!("{}.{pick}({})!", at(list), args.join(", "));
            let span = expr.span.shift(self.base);
            let mut diag = Diagnostic::warning(
                SORT_FOR_EXTREME.code,
                span,
                format!("this sorts a copy of the list to read one element: it is `{call}`"),
            )
            .with_label(list.span.shift(self.base), "the list")
            .with_help(format!(
                "write `{call}`: one O(n) pass with no copy instead of an O(n log n) sort, and it traps on an empty list just as the index does"
            ));
            diag.fixes.push(Fix {
                title: format!("use `{call}`"),
                edits: vec![(span, call)],
            });
            self.out.push(Finding {
                lint: &SORT_FOR_EXTREME,
                diag,
            });
        }
    }
    let mut v = Extremes {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

/// `xs` and its `by:` argument in `xs.sorted()` or `xs.sorted(by: f)`.
fn sorted_call(e: &Expr) -> Option<(&Expr, &[Arg])> {
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    let ExprKind::Field { base, name } = &callee.kind else {
        return None;
    };
    let by = match &args[..] {
        [] => true,
        [a] => a.name.as_ref().is_some_and(|n| n.text == "by"),
        _ => false,
    };
    (name.text == "sorted" && by).then_some((&**base, &args[..]))
}
