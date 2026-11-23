//! `float_sqrt_for_int` (L0529): `floor(sqrt(n))` is C++'s `(int)sqrt(n)`,
//! a float root that is off by one for some large `n`; `n.isqrt` is exact.
//! The operand is labelled so the fact table keeps the finding only on an
//! integer (`typed.rs`).

use crate::walk::Visitor;
use crate::{File, Finding, FLOAT_SQRT_FOR_INT};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::{Expr, ExprKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Isqrt<'a> {
        src: &'a str,
        base: u32,
        covered: Vec<Span>,
        out: Vec<Finding>,
    }
    impl Visitor for Isqrt<'_> {
        fn expr(&mut self, expr: &Expr) {
            let span = expr.span.shift(self.base);
            if self.covered.iter().any(|c| c.contains(span)) {
                return;
            }
            let Some(n) = call_of(expr, &["Int"])
                .and_then(rounded_root)
                .or_else(|| rounded_root(expr))
            else {
                return;
            };
            self.covered.push(span);
            let isqrt = format!("{}.isqrt", n.span.shift(self.base).text(self.src));
            let mut diag = Diagnostic::warning(
                FLOAT_SQRT_FOR_INT.code,
                span,
                format!("`{}` takes the root through `F64`", span.text(self.src)),
            )
            .with_label(n.span.shift(self.base), "an integer")
            .with_help(format!(
                "write `{isqrt}`, the exact integer square root; the float root is off by one for some values past 2**52"
            ));
            diag.fixes.push(Fix {
                title: format!("use `{isqrt}`"),
                edits: vec![(span, isqrt)],
            });
            self.out.push(Finding {
                lint: &FLOAT_SQRT_FOR_INT,
                diag,
            });
        }
    }
    let mut v = Isqrt {
        src: file.src,
        base: 0,
        covered: Vec::new(),
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

/// `n` in `floor(sqrt(n))`, `trunc(sqrt(F64(n)))`, `floor(n.sqrt)` or
/// `sqrt(F64(n)).floor` (each also inside `Int(...)`), when `n` is a name or
/// a field.
fn rounded_root(expr: &Expr) -> Option<&Expr> {
    let root = match &expr.kind {
        ExprKind::Field { base, name } if matches!(name.text.as_str(), "floor" | "trunc") => base,
        _ => call_of(expr, &["floor", "trunc"])?,
    };
    let n = match &root.kind {
        ExprKind::Field { base, name } if name.text == "sqrt" => base,
        _ => call_of(root, &["sqrt"])?,
    };
    let n = call_of(n, &["F64"]).unwrap_or(n);
    matches!(n.kind, ExprKind::Name(_) | ExprKind::Field { .. }).then_some(n)
}

/// The one unnamed argument of a call to one of `names`.
fn call_of<'e>(expr: &'e Expr, names: &[&str]) -> Option<&'e Expr> {
    let ExprKind::Call { callee, args } = &expr.kind else {
        return None;
    };
    let ExprKind::Name(name) = &callee.kind else {
        return None;
    };
    match &args[..] {
        [arg] if names.contains(&name.as_str()) && arg.name.is_none() && !arg.mutable => {
            Some(&arg.value)
        }
        _ => None,
    }
}
