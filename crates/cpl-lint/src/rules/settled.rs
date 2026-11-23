//! `always_true_bounds_check` (L0307): `if x >= 0:` or `n == 0` on declared
//! inputs whose ranges already settle it, so the test, or its branch, does
//! nothing.

use crate::{File, Finding, ALWAYS_TRUE_BOUNDS_CHECK};
use cpl_diag::{Diagnostic, Span};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    super::proven::hazards::proven_settled(file)
        .into_iter()
        .filter(|(at, _)| !super::wide_shift::in_test_fn(file, at.0))
        .map(|((start, end), always)| {
            let span = Span::new(start, end);
            let text = span.text(file.src);
            let (message, help) = if always {
                (
                    format!("`{text}` always holds: the declared bounds here already say so"),
                    "drop the test, or, if the statement allows values it rules out, widen the declared range so the test means something",
                )
            } else {
                (
                    format!("`{text}` never holds: the declared bounds here rule it out, so what it guards never runs"),
                    "drop the dead branch, or, if the statement allows that case, widen the declared range to include it",
                )
            };
            Finding {
                lint: &ALWAYS_TRUE_BOUNDS_CHECK,
                diag: Diagnostic::warning(ALWAYS_TRUE_BOUNDS_CHECK.code, span, message)
                    .with_help(help.to_string()),
            }
        })
        .collect()
}
