//! `int_overflow_at_bounds` (L0306): `a * b`, `a + b` or `a ** k` whose
//! operands' declared ranges let it leave `Int`, which traps (R0202). The
//! prover finds and checks an input that does.

use crate::{File, Finding, INT_OVERFLOW_AT_BOUNDS};
use cpl_diag::{Diagnostic, Span};

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
            let diag = Diagnostic::warning(
                INT_OVERFLOW_AT_BOUNDS.code,
                span,
                format!("`{text}` can leave `Int` (64 bits) at the declared bounds: with `{witness}` it does, and that traps"),
            )
            .with_help(format!(
                "its declared operands give `{}..={}`; widen an operand to `I128`, compute in `Mod<P>` if the answer is asked modulo `P`, or declare the tighter bounds the statement gives (R0202)",
                e.lo, e.hi
            ));
            Finding {
                lint: &INT_OVERFLOW_AT_BOUNDS,
                diag,
            }
        })
        .collect()
}
