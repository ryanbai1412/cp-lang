//! `noop_mod_reduce` (L0308): `k % n` where the declared bounds already put
//! `k` in `0..n`, so the remainder is `k` itself.

use crate::{File, Finding, NOOP_MOD_REDUCE};
use cpl_diag::{Diagnostic, Span};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    super::proven::hazards::proven_noop_mods(file)
        .into_iter()
        .filter(|(at, _)| !super::wide_shift::in_test_fn(file, at.0))
        .map(|((start, end), (x0, x1))| {
            let span = Span::new(start, end);
            let text = span.text(file.src);
            let x = Span::new(x0, x1).text(file.src);
            let mut diag = Diagnostic::warning(
                NOOP_MOD_REDUCE.code,
                span,
                format!("`{text}` is `{x}`: the declared bounds already put `{x}` below the modulus and at least 0"),
            )
            .with_help(format!(
                "write `{x}`; if `{x}` can reach the modulus after all, its declared range is too tight"
            ));
            diag.fix(format!("write `{x}`"), span, x.to_string());
            Finding {
                lint: &NOOP_MOD_REDUCE,
                diag,
            }
        })
        .collect()
}
