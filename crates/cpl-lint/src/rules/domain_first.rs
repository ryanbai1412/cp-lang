//! `domain_empty_first_last` (L0304): `D.first!` / `D.last!` where `D`'s
//! declared bounds let it be empty, so the unwrap traps on that input.

use super::bounds::{Bounds, Interval};
use crate::walk::{unparen, Visitor};
use crate::{File, Finding, DOMAIN_EMPTY_FIRST_LAST};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{Expr, ExprKind, StmtKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let bounds = Bounds::of(file);
    let declared = |e: &Expr| match &unparen(e).kind {
        ExprKind::Int(_) => bounds.expr(e),
        ExprKind::Name(name) if bounds.is_input(name) => bounds.expr(e),
        _ => None,
    };
    // Each domain the declared bounds let be empty, with its definition and
    // the end value that empties it.
    let mut empty: Vec<(String, String, String, i128)> = Vec::new();
    for (base, stmt) in file.top_level() {
        let StmtKind::Domain { name, def } = &stmt.kind else {
            continue;
        };
        let ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            step: None,
        } = &def.kind
        else {
            continue;
        };
        let (Some(a), Some(b)): (Option<Interval>, Option<Interval>) = (declared(lo), declared(hi))
        else {
            continue;
        };
        // The most `lo - hi` can be: from a relation when the two ends
        // have one, else from their declared ranges.
        let same = bounds.atom(lo).is_some() && bounds.atom(lo) == bounds.atom(hi);
        let gap = if same {
            0
        } else {
            bounds.diff(lo, hi).map_or(a.hi - b.lo, |d| d.hi)
        };
        let can_be_empty = if *inclusive { gap > 0 } else { gap >= 0 };
        if can_be_empty && matches!(unparen(hi).kind, ExprKind::Name(_)) {
            let text = |e: &Expr| e.span.shift(base).text(file.src).to_string();
            empty.push((name.text.clone(), text(def), text(hi), b.lo));
        }
    }
    if empty.is_empty() {
        return Vec::new();
    }
    struct Unwraps<'a> {
        src: &'a str,
        base: u32,
        empty: &'a [(String, String, String, i128)],
        out: Vec<Finding>,
    }
    impl Visitor for Unwraps<'_> {
        fn expr(&mut self, expr: &Expr) {
            let ExprKind::Unwrap(inner) = &expr.kind else {
                return;
            };
            let ExprKind::Field { base, name } = &unparen(inner).kind else {
                return;
            };
            if !matches!(name.text.as_str(), "first" | "last") {
                return;
            }
            let ExprKind::Name(domain) = &unparen(base).kind else {
                return;
            };
            let Some((d, def, hi, at)) = self.empty.iter().find(|(d, ..)| d == domain) else {
                return;
            };
            let span = expr.span.shift(self.base);
            let shown = span.text(self.src);
            let end = name.text.as_str();
            let diag = Diagnostic::warning(
                DOMAIN_EMPTY_FIRST_LAST.code,
                span,
                format!("`{d} = {def}` is empty when `{hi}` is {at}, and then `{shown}` traps"),
            )
            .with_help(format!(
                "`{d}.{end}` is `None` on an empty domain (R04.2.3): handle it with `if let v = {d}.{end}:`, or declare `{hi}` so `{d}` is never empty"
            ));
            self.out.push(Finding {
                lint: &DOMAIN_EMPTY_FIRST_LAST,
                diag,
            });
        }
    }
    let mut v = Unwraps {
        src: file.src,
        base: 0,
        empty: &empty,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}
