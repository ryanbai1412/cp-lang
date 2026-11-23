//! Choosing among an error's ranked fixes by re-checking, so `cpl fix`
//! applies, and the editor prefers, the candidate the types agree with
//! (`plan/domains/diagnostics/design/checked-fix-candidates.md`).

use cpl_diag::{Diagnostic, Fix};

/// Candidates re-checked for one error.
pub const MAX_CANDIDATES: usize = 8;
/// Re-checks for one `cpl fix` run, across every error.
pub const MAX_CHECKS: usize = 32;

/// The first of `diag.fixes` (at most `MAX_CANDIDATES`) after which `path`
/// checks clean, else the first after which no error starts on or before
/// `diag`'s line, else `fixes[0]`. One fix is returned with no re-check.
pub fn checked_fix<'d>(
    db: &mut cpl_check::Database,
    path: &str,
    text: &str,
    diag: &'d Diagnostic,
) -> Option<&'d Fix> {
    let mut budget = MAX_CANDIDATES;
    choose(db, path, text, diag, &mut budget)
}

fn choose<'d>(
    db: &mut cpl_check::Database,
    path: &str,
    text: &str,
    diag: &'d Diagnostic,
    budget: &mut usize,
) -> Option<&'d Fix> {
    let first = diag.fixes.first()?;
    if diag.fixes.len() == 1 {
        return Some(first);
    }
    let start = (diag.span.start as usize).min(text.len());
    let line_end = text[start..].find('\n').map_or(text.len(), |n| start + n);
    let mut on_line = None;
    for fix in diag.fixes.iter().take(MAX_CANDIDATES) {
        if *budget == 0 {
            break;
        }
        *budget -= 1;
        let alone = Diagnostic {
            fixes: vec![fix.clone()],
            ..diag.clone()
        };
        let (fixed, _) = cpl_lint::apply_fixes(text, std::slice::from_ref(&alone));
        let errors = errors_in(db, path, &fixed);
        if errors.is_empty() {
            return Some(fix);
        }
        let grown = fixed.len() as isize - text.len() as isize;
        let limit = (line_end as isize + grown).max(0) as u32;
        if on_line.is_none() && errors.iter().all(|e| e.span.start > limit) {
            on_line = Some(fix);
        }
    }
    on_line.or(Some(first))
}

/// `errors` with the checked fix moved first in each, so applying first
/// fixes applies the checked ones, sharing `MAX_CHECKS` re-checks.
pub fn rank_checked(
    db: &mut cpl_check::Database,
    path: &str,
    text: &str,
    errors: &mut [Diagnostic],
    budget: &mut usize,
) {
    for error in errors.iter_mut() {
        let chosen = choose(db, path, text, error, budget).cloned();
        if let Some(fix) = chosen {
            let fixes = &mut error.fixes;
            if let Some(at) = fixes.iter().position(|f| *f == fix) {
                fixes[..=at].rotate_right(1);
            }
        }
    }
}

/// The errors the front end reports for `text` as `path`.
pub fn errors_in(db: &mut cpl_check::Database, path: &str, text: &str) -> Vec<Diagnostic> {
    db.set_file(path, text);
    let analysis = db.analyze(path).expect("the file was just set");
    let mut out = cpl_ir::check(&analysis);
    out.retain(|d| d.severity == cpl_diag::Severity::Error);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpl_diag::{codes, Span};

    /// An error at the start of `text` whose fixes insert each of `pieces`
    /// after `let x = `.
    fn diag(pieces: &[&str]) -> Diagnostic {
        let mut d = Diagnostic::error(codes::UNCLOSED_DELIMITER, Span::new(0, 3), "test");
        for piece in pieces {
            d.fix(format!("insert {piece}"), Span::point(8), *piece);
        }
        d
    }

    fn chosen(text: &str, d: &Diagnostic) -> String {
        let mut db = cpl_check::Database::new();
        checked_fix(&mut db, "a.cp", text, d).unwrap().title.clone()
    }

    #[test]
    fn the_first_candidate_that_checks_clean_wins() {
        let text = "let x = \nprint(x)\n";
        assert_eq!(chosen(text, &diag(&["zz", "1", "2"])), "insert 1");
    }

    #[test]
    fn else_the_first_that_clears_the_line_else_the_first() {
        let text = "let x = \nlet y = \nprint(x, y)\n";
        assert_eq!(chosen(text, &diag(&["zz", "1"])), "insert 1");
        assert_eq!(chosen(text, &diag(&["zz", "yy"])), "insert zz");
    }

    #[test]
    fn only_the_first_eight_candidates_are_tried() {
        let text = "let x = \nprint(x)\n";
        let mut pieces = vec!["zz"; MAX_CANDIDATES];
        pieces.push("1");
        assert_eq!(chosen(text, &diag(&pieces)), "insert zz");
    }

    #[test]
    fn one_fix_is_kept_without_a_check() {
        // `zz` would not check; one candidate is returned as is.
        assert_eq!(chosen("let x = \nprint(x)\n", &diag(&["zz"])), "insert zz");
    }

    #[test]
    fn a_spent_budget_keeps_the_ranked_order() {
        let text = "let x = \nprint(x)\n";
        let mut db = cpl_check::Database::new();
        let mut errors = vec![diag(&["zz", "1"])];
        rank_checked(&mut db, "a.cp", text, &mut errors, &mut 0);
        assert_eq!(errors[0].fixes[0].title, "insert zz");
        rank_checked(&mut db, "a.cp", text, &mut errors, &mut MAX_CHECKS.clone());
        assert_eq!(errors[0].fixes[0].title, "insert 1");
    }
}
