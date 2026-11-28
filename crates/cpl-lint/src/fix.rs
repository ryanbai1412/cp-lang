//! The one checked fix path. `cpl fix`, `cpl lint --fix`, `cpl migrate`, the
//! editor and the playground apply fixes here, in rounds: a fix stands only
//! when the text it leaves reports no error kind more often than before and
//! the finding it fixes fires less (reparsed for the parser's rewrites,
//! rechecked for the rest).

use crate::{lint_analysis, Config};
use cpl_diag::{codes, Diagnostic, Severity};
use cpl_facade::Layer;

/// What a text reports to a fix round: the findings to fix, and the errors
/// a fix must not add to.
pub type Report<'a> = dyn FnMut(&str) -> Reported + 'a;

/// The findings a text reports, and its errors.
pub type Reported = (Vec<Diagnostic>, Vec<Diagnostic>);

/// `text` with `report`'s findings fixed in checked rounds until none
/// applies, and how many fixes were applied. A round splices every fix at
/// once and reports the result once; when that fails, the fixes the report
/// does not blame are tried, then the first alone, so `report` runs at most
/// three times a round, never once per fix. Blamed fixes wait a round.
pub fn fix_rounds(text: &str, report: &mut Report) -> (String, usize) {
    let mut text = text.to_string();
    let (mut total, mut refused) = (0, Vec::new());
    let mut before = report(&text);
    for _ in 0..8 {
        let found: Vec<_> = (before.0.iter())
            .filter(|d| !d.fixes.is_empty() && !refused.contains(&fix_key(d)))
            .cloned()
            .collect();
        if found.is_empty() {
            break;
        }
        if let Some((next, n, after)) = fix_round(&text, &before, found, report, &mut refused) {
            total += n;
            text = next;
            before = after;
        }
    }
    (text, total)
}

/// `found`'s fixes on `text` that verify against `before`, with how many and
/// what the result reports; `None` after refusing a fix that fails alone.
fn fix_round(
    text: &str,
    before: &Reported,
    mut found: Vec<Diagnostic>,
    report: &mut Report,
    refused: &mut Vec<String>,
) -> Option<(String, usize, Reported)> {
    for _ in 0..3 {
        let (fixed, n) = cpl_diag::splice_fixes(text, &found);
        if n == 0 {
            break;
        }
        let after = report(&fixed);
        if verified(before, &after, &found) {
            return Some((fixed, n, after));
        }
        let kept = unblamed(&fixed, before, &after, &found);
        if !kept.is_empty() && kept.len() < found.len() {
            found = kept;
        } else if found.len() > 1 {
            found.truncate(1);
        } else {
            break;
        }
    }
    refused.push(fix_key(&found[0]));
    None
}

/// What tells one finding's fix from another's across rounds, whose spans
/// move as earlier fixes land.
fn fix_key(d: &Diagnostic) -> String {
    let edits: Vec<_> = d.fixes[0].edits.iter().map(|(_, to)| to.as_str()).collect();
    format!("{:?} {} {edits:?}", d.code, d.message)
}

/// The findings of `applied` that `after` does not blame: their code fires
/// less, and no error `fixed` reports more often is on a line their fix
/// edits.
fn unblamed(
    fixed: &str,
    before: &Reported,
    after: &Reported,
    applied: &[Diagnostic],
) -> Vec<Diagnostic> {
    let count = |ds: &[Diagnostic], d: &Diagnostic| ds.iter().filter(|e| e.code == d.code).count();
    let new_errors: Vec<_> = (after.1.iter())
        .filter(|e| e.severity == Severity::Error && count(&after.1, e) > count(&before.1, e))
        .collect();
    let shift = |at: u32| -> i64 {
        let edits = applied.iter().flat_map(|d| &d.fixes[0].edits);
        edits
            .filter(|(span, _)| span.end <= at)
            .map(|(span, to)| to.len() as i64 - i64::from(span.end - span.start))
            .sum()
    };
    let line = |s: &str, at: i64| {
        let at = (at.max(0) as usize).min(s.len());
        let start = s[..at].rfind('\n').map_or(0, |i| i + 1);
        let end = s[at..].find('\n').map_or(s.len(), |i| at + i);
        (start, end)
    };
    let on_edited_line = |d: &Diagnostic| {
        d.fixes[0].edits.iter().any(|(span, to)| {
            let at = i64::from(span.start) + shift(span.start);
            let (lo, hi) = (line(fixed, at).0, line(fixed, at + to.len() as i64).1);
            new_errors
                .iter()
                .any(|e| (e.span.start as usize) <= hi && lo <= e.span.end as usize)
        })
    };
    (applied.iter())
        .filter(|d| count(&after.0, d) < count(&before.0, d) && !on_edited_line(d))
        .cloned()
        .collect()
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
fn verified(before: &Reported, after: &Reported, applied: &[Diagnostic]) -> bool {
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
    db: &mut cpl_facade::Session,
    key: &str,
    src: &str,
) -> (String, usize) {
    let codemods = [
        codes::REMOVED_SYNTAX,
        codes::NOT_CALLABLE,
        codes::WRONG_ARG_COUNT,
    ];
    let out = fix_rounds(src, &mut |text| {
        *db = db.with_file(key, text);
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
    *db = db.with_file(key, out.0.as_str());
    out
}

/// `text` with the lint fixes `cpl lint --fix` would make under `config`,
/// and how many; a fix is rechecked against the file's errors up to `upto`.
pub fn apply_lint_fixes(
    db: &mut cpl_facade::Session,
    path: &str,
    text: String,
    config: &Config,
    upto: Layer,
) -> (String, usize) {
    apply_lint_fixes_where(db, path, text, config, upto, &|_| true)
}

/// [`apply_lint_fixes`] with only the findings `keep` takes.
pub fn apply_lint_fixes_where(
    db: &mut cpl_facade::Session,
    path: &str,
    text: String,
    config: &Config,
    upto: Layer,
    keep: &dyn Fn(&Diagnostic) -> bool,
) -> (String, usize) {
    let out = fix_rounds(&text, &mut |text| {
        let analysis = db.analyze_text(path, text);
        let mut findings = lint_analysis(&analysis, config);
        findings.retain(|d| keep(d));
        (findings, db.diagnostics(path, upto))
    });
    *db = db.with_file(path, out.0.as_str());
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
        &mut cpl_facade::Session::default(),
        &key,
        src,
        config,
        Layer::Built,
    )
}

/// [`fix`] on the file `key` in a warm `db`, leaving `db` holding the result.
pub fn fix_in(
    db: &mut cpl_facade::Session,
    key: &str,
    src: &str,
    config: &Config,
    upto: Layer,
) -> (String, usize) {
    let (src, total) = fix_removed_syntax(src);
    if !cpl_syntax::parse_file(&src).diagnostics.is_empty() {
        return (src, total);
    }
    let (src, checked) = fix_checked_removed_syntax_in(db, key, &src);
    let (src, linted) = apply_lint_fixes(db, key, src, config, upto);
    (src, total + checked + linted)
}
