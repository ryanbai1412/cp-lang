//! The lints the checker emits, the 0.3 table of `spec/diagnostics.md`.
//! The checker finds them and `cpl-diag` explains them; this table only
//! names them, so `cp.toml [lints]`, `with lint(name: allow)` and
//! `@allow(name)` set their level and `cpl lint` reports them.

use crate::{with_parsed, Allows, Config, Level};
use cpl_diag::{Code, Diagnostic, Severity};
use std::path::Path;

/// One spec name of a lint the checker emits.
pub struct CheckerLint {
    pub code: Code,
    pub name: &'static str,
}

const fn named(code: &'static str, name: &'static str) -> CheckerLint {
    CheckerLint {
        code: Code(code),
        name,
    }
}

/// Every name the spec gives a checker lint; L0608 has two.
pub const CHECKER_LINTS: &[CheckerLint] = &[
    named("L0601", "same_place_read_write"),
    named("L0602", "chain_walked_twice"),
    named("L0603", "map_recomputed"),
    named("L0604", "mut_generator_rewalked"),
    named("L0605", "literal_slice_index"),
    named("L0606", "wide_array"),
    named("L0607", "padding"),
    named("L0608", "parens_on_o1"),
    named("L0608", "missing_parens"),
    named("L0609", "cache_field_written_outside"),
];

pub(crate) fn checker_lint_named(name: &str) -> Option<&'static CheckerLint> {
    CHECKER_LINTS.iter().find(|lint| lint.name == name)
}

/// The checker's lints on `src`, at the levels `config` and the file's own
/// `with lint(...)` and `@allow(...)` set. Empty when `src` does not parse.
pub fn checker_lints(path: Option<&Path>, src: &str, config: &Config) -> Vec<Diagnostic> {
    checked(path, src, config).lints
}

/// What the checker says about `src`: its lints, as `checker_lints` gives
/// them, and how many errors it reports, which lints cannot see past.
#[derive(Default)]
pub struct Checked {
    pub lints: Vec<Diagnostic>,
    pub errors: usize,
}

pub fn checked(path: Option<&Path>, src: &str, config: &Config) -> Checked {
    let Some(regions) = with_parsed(src, |file| Some(Allows::collect(file).regions)) else {
        return Checked::default();
    };
    let key = path
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "<lint>.cp".to_string());
    let mut db = cpl_check::Database::new();
    db.set_file(&key, src);
    let Some(analysis) = db.analyze(&key) else {
        return Checked::default();
    };
    let mut out = Vec::new();
    let mut errors = 0;
    for diag in &analysis.diagnostics {
        if diag.severity == Severity::Error && !CHECKER_LINTS.iter().any(|l| l.code == diag.code) {
            errors += 1;
        }
        let mut names = CHECKER_LINTS.iter().filter(|lint| lint.code == diag.code);
        let allowed = names.any(|lint| {
            regions
                .iter()
                .any(|(span, name)| *name == lint.name && span.contains(diag.span))
        });
        let level = config.checker_level(diag.code);
        if allowed || level == Level::Allow || !CHECKER_LINTS.iter().any(|l| l.code == diag.code) {
            continue;
        }
        let mut diag = diag.clone();
        if level == Level::Deny {
            diag.severity = Severity::Error;
        }
        out.push(diag);
    }
    Checked { lints: out, errors }
}
