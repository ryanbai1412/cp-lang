//! The lints the checker emits, the 0.3 table of `spec/diagnostics.md`.
//! The checker finds them and `cpl-diag` explains them; this table only
//! names them, so `cp.toml [lints]` and `with lint(name: allow)` set
//! their level and `cpl lint` reports them.

use crate::{Allows, Config, Group, Level};
use cpl_diag::{Code, Diagnostic, Severity};

/// One spec name of a lint the checker emits.
pub struct CheckerLint {
    pub code: Code,
    pub name: &'static str,
    pub group: Group,
}

const fn named(code: &'static str, name: &'static str, group: Group) -> CheckerLint {
    CheckerLint {
        code: Code(code),
        name,
        group,
    }
}

/// Every name the spec gives a checker lint; L0608 has two.
pub const CHECKER_LINTS: &[CheckerLint] = &[
    named("L0601", "same_place_read_write", Group::Correctness),
    named("L0602", "chain_walked_twice", Group::Perf),
    named("L0603", "map_recomputed", Group::Perf),
    named("L0604", "mut_generator_rewalked", Group::Perf),
    named("L0605", "literal_slice_index", Group::Correctness),
    named("L0606", "wide_array", Group::Perf),
    named("L0607", "padding", Group::Perf),
    named("L0608", "parens_on_o1", Group::Style),
    named("L0608", "missing_parens", Group::Style),
    named("L0609", "cache_field_written_outside", Group::Correctness),
    named("L0610", "mut_param_unused", Group::Style),
];

pub(crate) fn checker_lint_named(name: &str) -> Option<&'static CheckerLint> {
    CHECKER_LINTS.iter().find(|lint| lint.name == name)
}

/// The checker's lints on `src`, at the levels `config` and the file's own
/// `with lint(...)` sets. Empty when `src` does not parse.
#[cfg(test)]
pub(crate) fn checker_lints(
    path: Option<&std::path::Path>,
    src: &str,
    config: &Config,
) -> Vec<Diagnostic> {
    checked(path, src, config).lints
}

/// What the checker says about `src`: its lints, as `checker_lints` gives
/// them, and the errors it reports, which lints cannot see past.
#[derive(Default)]
pub struct Checked {
    pub lints: Vec<Diagnostic>,
    pub errors: usize,
    pub problems: Vec<Diagnostic>,
}

#[cfg(test)]
pub(crate) fn checked(path: Option<&std::path::Path>, src: &str, config: &Config) -> Checked {
    let key = path
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "<lint>.cp".to_string());
    let mut db = cpl_check::Database::new();
    db.set_file(&key, src);
    let Some(analysis) = db.analyze(&key) else {
        return Checked::default();
    };
    checked_analysis(&analysis, &analysis.diagnostics, config)
}

/// [`checked`] on a warm `analysis`, sorting what `cpl check` reported for
/// it (`reported`, the checker's diagnostics or more) into lints at their
/// levels and the errors lints can't see past.
pub fn checked_analysis(
    analysis: &cpl_check::Analysis,
    reported: &[Diagnostic],
    config: &Config,
) -> Checked {
    if analysis.has_syntax_errors {
        return Checked::default();
    }
    let items: Vec<_> = (analysis.items.iter())
        .map(|i| (i.offset, &i.syntax.parsed))
        .collect();
    let file = crate::File {
        src: &analysis.text,
        items: &items,
        facts: None,
    };
    let regions = Allows::collect(&file).regions;
    let mut out = Vec::new();
    let mut problems = Vec::new();
    for diag in reported {
        if diag.severity == Severity::Error && !CHECKER_LINTS.iter().any(|l| l.code == diag.code) {
            problems.push(diag.clone());
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
    Checked {
        lints: out,
        errors: problems.len(),
        problems,
    }
}
