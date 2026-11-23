//! `int_shift_overflow_at_bounds` (L0302): `x << k` where the declared
//! bounds let `k` shift `x` out of `Int`, which traps (R0202) on that input.

use crate::{File, Finding, INT_SHIFT_OVERFLOW_AT_BOUNDS};
use cpl_diag::{Diagnostic, Span};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    super::proven_shifts(file)
        .into_iter()
        .filter(|(op, ..)| !in_test_fn(file, op.0))
        .map(|(_, (start, end), hi, safe)| {
            let k = Span::new(start, end);
            let text = k.text(file.src);
            let diag = Diagnostic::warning(
                INT_SHIFT_OVERFLOW_AT_BOUNDS.code,
                k,
                format!("this shift leaves `Int` when `{text}` is above {safe}, and its bounds allow {hi}"),
            )
            .with_help(format!(
                "if the statement keeps `{text}` at most {safe}, declare that range; otherwise shift a `U64` (`U64(1) << {text}`) or keep the bits in a `Bitset` (`std.bitset`), so the largest input does not trap (R0202)"
            ))
            .with_witness(super::witness(text, hi));
            Finding {
                lint: &INT_SHIFT_OVERFLOW_AT_BOUNDS,
                diag,
            }
        })
        .collect()
}

/// Whether `at` is inside a top-level `@alt`, `@check` or `@gen` fn, which
/// runs on small generated inputs, not at the declared bounds.
pub(super) fn in_test_fn(file: &File, at: u32) -> bool {
    file.top_level().any(|(base, stmt)| {
        let span = stmt.span.shift(base);
        let text = span.text(file.src).trim_start();
        span.start <= at
            && at < span.end
            && ["@alt", "@check", "@gen"]
                .iter()
                .any(|d| text.starts_with(d))
    })
}
