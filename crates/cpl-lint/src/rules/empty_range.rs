//! `reversed_range_literal` (L0303): `for i in n..0`, a range whose declared
//! start is always above its end, so the loop never runs.

use crate::{File, Finding, REVERSED_RANGE_LITERAL};
use cpl_diag::{Diagnostic, Span};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    super::proven_empty_ranges(file)
        .into_iter()
        .map(|(start, end)| {
            let span = Span::new(start, end);
            let text = span.text(file.src);
            let (lo, hi, _) = text
                .split_once("..=")
                .map(|(a, b)| (a, b, "..="))
                .or_else(|| text.split_once("..").map(|(a, b)| (a, b, "..")))
                .unwrap_or((text, "", ".."));
            let (lo, hi) = (lo.trim(), hi.trim());
            let diag = Diagnostic::warning(
                REVERSED_RANGE_LITERAL.code,
                span,
                format!("`{text}` is empty for every input, so this loop never runs: `{lo}` is always above `{hi}`"),
            )
            .with_help(format!(
                "a range only counts up; to count down, reverse the range you mean: `({hi}..{lo}).rev` runs `{lo} - 1` down to `{hi}`, `({hi}..={lo}).rev` runs `{lo}` down to `{hi}`"
            ));
            Finding {
                lint: &REVERSED_RANGE_LITERAL,
                diag,
            }
        })
        .collect()
}
