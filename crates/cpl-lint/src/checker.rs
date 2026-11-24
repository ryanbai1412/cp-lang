//! The lints the checker emits, the 0.3 table of `spec/diagnostics.md`.
//! The checker finds them and `cpl-diag` explains them; this table only
//! names them, so `cp.toml [lints]` and `with lint(name: allow)` set
//! their level and `cpl lint` reports them.

use crate::{Allows, Config, Group, Level, CHECKER_LINTS};
use cpl_diag::{Code, Diagnostic, Severity};

/// One spec name of a lint the checker emits.
pub struct CheckerLint {
    pub code: Code,
    pub name: &'static str,
    pub group: Group,
}

pub(crate) const fn named(code: &'static str, name: &'static str, group: Group) -> CheckerLint {
    CheckerLint {
        code: Code(code),
        name,
        group,
    }
}

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
    let mut db = cpl_check::Session::new();
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
    let regions = allowed_regions(analysis);
    let mut out = Vec::new();
    let mut problems = Vec::new();
    for diag in reported {
        match level_in(&regions, diag, config) {
            None if diag.severity == Severity::Error => problems.push(diag.clone()),
            Some(Some(diag)) => out.push(diag),
            _ => {}
        }
    }
    Checked {
        lints: out,
        errors: problems.len(),
        problems,
    }
}

/// `reported` as `cpl check` and the editor show it: the checker's lints at
/// the levels `config` and the file's `with lint(...)` set, the rest as is.
pub fn at_levels(
    analysis: &cpl_check::Analysis,
    reported: &[Diagnostic],
    config: &Config,
) -> Vec<Diagnostic> {
    let regions = allowed_regions(analysis);
    reported
        .iter()
        .filter_map(|diag| level_in(&regions, diag, config).unwrap_or(Some(diag.clone())))
        .collect()
}

/// The spans `with lint(name: allow)` covers in `analysis`'s file.
fn allowed_regions(analysis: &cpl_check::Analysis) -> Vec<(cpl_diag::Span, &'static str)> {
    if analysis.has_syntax_errors {
        return Vec::new();
    }
    let items: Vec<_> = (analysis.items.iter())
        .map(|i| (i.offset, &i.syntax.parsed))
        .collect();
    let file = crate::File {
        src: &analysis.text,
        items: &items,
        facts: None,
        proven: &Default::default(),
    };
    Allows::collect(&file, &[]).regions
}

/// `None` when `diag` is no checker lint; else the lint at its level, or
/// `Some(None)` when it is allowed.
fn level_in(
    regions: &[(cpl_diag::Span, &'static str)],
    diag: &Diagnostic,
    config: &Config,
) -> Option<Option<Diagnostic>> {
    let mut names = CHECKER_LINTS
        .iter()
        .filter(|lint| lint.code == diag.code)
        .peekable();
    names.peek()?;
    let allowed = names.any(|lint| {
        regions
            .iter()
            .any(|(span, name)| *name == lint.name && span.contains(diag.span))
    });
    let level = config.checker_level(diag.code);
    if allowed || level == Level::Allow {
        return Some(None);
    }
    let mut diag = diag.clone();
    if level == Level::Deny {
        diag.severity = Severity::Error;
    }
    Some(Some(diag))
}
