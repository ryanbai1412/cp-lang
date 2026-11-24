//! The one checked fix path. `cpl fix`, `cpl lint --fix`, `cpl migrate`, the
//! editor and the playground apply fixes here, in rounds: a fix stands only
//! when the text it leaves reports no error kind more often than before and
//! the finding it fixes fires less (reparsed for the parser's rewrites,
//! rechecked for the rest).

use crate::{lint_analysis, Config, Errors};
use cpl_diag::{codes, Diagnostic, Severity};

/// What a text reports to a fix round: the findings to fix, and the errors
/// a fix must not add to.
pub type Report<'a> = dyn FnMut(&str) -> (Vec<Diagnostic>, Vec<Diagnostic>) + 'a;

/// `text` with `report`'s findings fixed in checked rounds until none
/// applies, and how many fixes were applied.
pub fn fix_rounds(text: &str, report: &mut Report) -> (String, usize) {
    let mut text = text.to_string();
    let mut total = 0;
    for _ in 0..8 {
        let before = report(&text);
        let (next, n) = checked_fixes(&text, &before, report);
        if n == 0 || next == text {
            break;
        }
        total += n;
        text = next;
    }
    (text, total)
}

/// `text` with the fixes of `before`'s findings that verify: all at once
/// when they do together, else each that does alone, else the first such.
pub fn checked_fixes(
    text: &str,
    before: &(Vec<Diagnostic>, Vec<Diagnostic>),
    report: &mut Report,
) -> (String, usize) {
    let found: Vec<_> = before
        .0
        .iter()
        .filter(|d| !d.fixes.is_empty())
        .cloned()
        .collect();
    let mut holds = |fixed: &str, applied: &[Diagnostic]| verified(before, &report(fixed), applied);
    let mut safe = accepted(text, &found, &mut holds);
    if safe.len() > 1 && !holds_for(text, &safe, &mut holds) {
        safe.truncate(1);
    }
    cpl_diag::splice_fixes(text, &safe)
}

type Holds<'a> = dyn FnMut(&str, &[Diagnostic]) -> bool + 'a;

/// The findings of `found` whose fixes `holds` accepts: all when it accepts
/// them together, else each it accepts alone.
pub(crate) fn accepted(text: &str, found: &[Diagnostic], holds: &mut Holds) -> Vec<Diagnostic> {
    if holds_for(text, found, holds) {
        return found.to_vec();
    }
    let alone = |d: &&Diagnostic| holds_for(text, std::slice::from_ref(*d), holds);
    found.iter().filter(alone).cloned().collect()
}

fn holds_for(text: &str, found: &[Diagnostic], holds: &mut Holds) -> bool {
    let (fixed, n) = cpl_diag::splice_fixes(text, found);
    n > 0 && holds(&fixed, found)
}

/// Whether `after` has no error code more often than `before`, and each
/// `applied` finding's code fewer times.
fn verified(
    before: &(Vec<Diagnostic>, Vec<Diagnostic>),
    after: &(Vec<Diagnostic>, Vec<Diagnostic>),
    applied: &[Diagnostic],
) -> bool {
    let count = |ds: &[Diagnostic], d: &Diagnostic| ds.iter().filter(|e| e.code == d.code).count();
    let errors = |ds: &[Diagnostic]| -> Vec<Diagnostic> {
        ds.iter()
            .filter(|d| d.severity == Severity::Error)
            .cloned()
            .collect()
    };
    let (was, now) = (errors(&before.1), errors(&after.1));
    now.iter().all(|e| count(&now, e) <= count(&was, e))
        && applied
            .iter()
            .all(|d| count(&after.0, d) < count(&before.0, d))
}

/// The removed spellings the parser offers a rewrite for (`where` as a
/// filter, `--- multitest ---`), each checked by a reparse. Parser recovery
/// guesses, such as an inserted `:`, are not among them.
pub fn fix_removed_syntax(src: &str) -> (String, usize) {
    fix_rounds(src, &mut |text| {
        let all = cpl_syntax::parse_file(text).diagnostics;
        let codemod = |d: &&Diagnostic| {
            d.code == codes::REMOVED_SYNTAX
                || (d.code == codes::UNKNOWN_CHAR && d.message == "CP Lang concatenates with `+`")
        };
        (all.iter().filter(codemod).cloned().collect(), all)
    })
}

/// Type-directed rewrites the checker reports for `key` in a warm `db`,
/// each rechecked: removed syntax (such as `Range.lo`/`.hi` ->
/// `.start`/`.end`, leaving user struct fields named `lo`/`hi` untouched)
/// and a getter called with `()`.
pub fn fix_checked_removed_syntax_in(
    db: &mut cpl_check::Session,
    key: &str,
    src: &str,
) -> (String, usize) {
    let codemods = [
        codes::REMOVED_SYNTAX,
        codes::NOT_CALLABLE,
        codes::WRONG_ARG_COUNT,
    ];
    let out = fix_rounds(src, &mut |text| {
        db.set_file(key, text);
        let all = db
            .analyze(key)
            .map(|a| a.diagnostics.clone())
            .unwrap_or_default();
        (
            all.iter()
                .filter(|d| codemods.contains(&d.code))
                .cloned()
                .collect(),
            all,
        )
    });
    db.set_file(key, out.0.as_str());
    out
}

/// `text` with the lint fixes `cpl lint --fix` would make under `config`,
/// and how many; a fix is rechecked against the errors `check` reports.
pub fn apply_lint_fixes(
    db: &mut cpl_check::Session,
    path: &str,
    text: String,
    config: &Config,
    check: &Errors,
) -> (String, usize) {
    apply_lint_fixes_where(db, path, text, config, check, &|_| true)
}

/// [`apply_lint_fixes`] with only the findings `keep` takes.
pub fn apply_lint_fixes_where(
    db: &mut cpl_check::Session,
    path: &str,
    text: String,
    config: &Config,
    check: &Errors,
    keep: &dyn Fn(&Diagnostic) -> bool,
) -> (String, usize) {
    let out = fix_rounds(&text, &mut |text| {
        let analysis = db.analyze_text(path, text);
        let mut findings = lint_analysis(&analysis, config);
        findings.retain(|d| keep(d));
        (findings, check(&analysis))
    });
    db.set_file(path, out.0.as_str());
    out
}

/// What `cpl fix` makes of a source: removed-syntax rewrites (parser, then
/// type-directed checker rewrites), then every lint fix, each to a fixed point.
#[cfg(test)]
pub(crate) fn fix(src: &str, config: &Config) -> (String, usize) {
    fix_at(None, src, config)
}

/// Like [`fix`], resolving imports relative to `path` when given.
#[cfg(test)]
pub(crate) fn fix_at(
    path: Option<&std::path::Path>,
    src: &str,
    config: &Config,
) -> (String, usize) {
    let key = path
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "<fix>.cp".to_string());
    fix_in(
        &mut cpl_check::Session::new(),
        &key,
        src,
        config,
        &cpl_ir::check,
    )
}

/// [`fix`] on the file `key` in a warm `db`, leaving `db` holding the result.
pub fn fix_in(
    db: &mut cpl_check::Session,
    key: &str,
    src: &str,
    config: &Config,
    check: &Errors,
) -> (String, usize) {
    let (src, total) = fix_removed_syntax(src);
    if !cpl_syntax::parse_file(&src).diagnostics.is_empty() {
        return (src, total);
    }
    let (src, checked) = fix_checked_removed_syntax_in(db, key, &src);
    let (src, linted) = apply_lint_fixes(db, key, src, config, check);
    (src, total + checked + linted)
}
