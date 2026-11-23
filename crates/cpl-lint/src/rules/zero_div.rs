//! `int_div_possible_zero` (L0599): `a / b` or `a % b` where the declared
//! bounds let `b` be 0, which traps (R0203) on that input.

use crate::{File, Finding, INT_DIV_POSSIBLE_ZERO};
use cpl_diag::{Diagnostic, Span};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    super::proven_divisors(file)
        .into_iter()
        .filter(|(op, ..)| !super::wide_shift::in_test_fn(file, op.0))
        .map(|(op, (start, end), lo, hi)| {
            let divisor = Span::new(start, end);
            let text = divisor.text(file.src);
            let op_text = Span::new(op.0, op.1).text(file.src);
            let narrow = if lo == 0 {
                format!(" or declare it `1..={hi}`")
            } else {
                String::new()
            };
            let diag = Diagnostic::warning(
                INT_DIV_POSSIBLE_ZERO.code,
                divisor,
                format!("`{text}` can be 0 here (its bounds give `{lo}..={hi}`), and `{op_text}` by 0 traps"),
            )
            .with_help(format!(
                "guard it with `if {text} != 0:`{narrow}, so the input that makes it 0 is handled instead of trapping (R0203)"
            ))
            .with_witness(super::witness(text, 0));
            Finding {
                lint: &INT_DIV_POSSIBLE_ZERO,
                diag,
            }
        })
        .collect()
}
