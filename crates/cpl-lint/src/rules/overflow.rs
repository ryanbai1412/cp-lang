//! `int_overflow_at_bounds` (L0306): `a * b`, `a + b` or `a ** k` whose
//! operands' declared ranges let it leave `Int`, which traps (R0202). The
//! prover finds and checks an input that does.

use crate::{File, Finding, INT_OVERFLOW_AT_BOUNDS};
use cpl_diag::msgs::lint_rules;
use cpl_diag::Span;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let leaves: Vec<_> = super::proven_expr_ranges(file)
        .into_iter()
        .filter(|e| e.overflow.is_some() && !super::wide_shift::in_test_fn(file, e.op.0))
        .collect();
    // Only the innermost: an expression around one that leaves `Int` says
    // nothing new.
    let inner = |e: &crate::ProvenExpr| {
        !leaves
            .iter()
            .any(|o| o.expr != e.expr && e.expr.0 <= o.expr.0 && o.expr.1 <= e.expr.1)
    };
    leaves
        .iter()
        .filter(|e| inner(e))
        .map(|e| {
            let span = Span::new(e.expr.0, e.expr.1);
            let text = span.text(file.src);
            let witness = e.overflow.as_deref().unwrap_or_default();
            let diag = lint_rules::INT_OVERFLOW_AT_BOUNDS.warning(span, &[&text, &witness])
            .with_help(format!(
                "its declared operands give `{}..={}`; widen an operand to `I128`, compute in `Mod<P>` if the answer is asked modulo `P`, or declare the tighter bounds the statement gives (R0202)",
                e.lo, e.hi
            ))
            .with_witness(pairs(witness));
            Finding {
                lint: &INT_OVERFLOW_AT_BOUNDS,
                diag,
            }
        })
        .collect()
}

/// The prover's `a = 5, b = 7` as names and values.
fn pairs(witness: &str) -> Vec<(String, i128)> {
    witness
        .split(", ")
        .filter_map(|pair| {
            let (name, value) = pair.split_once(" = ")?;
            Some((name.trim().to_string(), value.trim().parse().ok()?))
        })
        .collect()
}
