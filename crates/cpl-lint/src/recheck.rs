//! Findings that stand only if their fix leaves the checker's verdict
//! alone. `redundant_prelude_import` (L0208) says a `use` changes nothing;
//! the file is checked with the `use` dropped, and the finding goes when
//! any error appears, disappears or changes. `index_only_loop` (L0505)
//! goes the same way when the loop item it walks is lent and the body
//! stores it (E0603).

use crate::{INDEX_ONLY_LOOP, REDUNDANT_PRELUDE_IMPORT};

const RECHECKED: [cpl_diag::Code; 2] = [REDUNDANT_PRELUDE_IMPORT.code, INDEX_ONLY_LOOP.code];
use cpl_diag::{Diagnostic, Severity};

/// The errors of `src` as `(code, message)`, in a stable order: spans move
/// when a line goes, what is reported does not.
fn verdict(db: &mut cpl_check::Session, src: &str) -> Vec<(String, String)> {
    db.set_file("<lint>.cp", src);
    errors(db.diagnostics("<lint>.cp"))
}

fn errors(diags: Vec<Diagnostic>) -> Vec<(String, String)> {
    let mut out: Vec<_> = diags
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| (d.code.0.to_string(), d.message))
        .collect();
    out.sort();
    out
}

/// `diags` less the [`RECHECKED`] findings whose fix changes how `src` checks.
pub(crate) fn recheck(diags: &mut Vec<Diagnostic>, src: &str) {
    CHECKER.with_borrow_mut(|db| keep_if_unchanged(diags, src, |text| verdict(db, text)));
}

thread_local! {
    /// Kept between rechecks, so the modules a file imports are checked once
    /// per thread rather than on every keystroke.
    static CHECKER: std::cell::RefCell<cpl_check::Session> =
        std::cell::RefCell::new(cpl_check::Session::new());
}

pub(crate) fn keep_if_unchanged(
    diags: &mut Vec<Diagnostic>,
    src: &str,
    mut check: impl FnMut(&str) -> Vec<(String, String)>,
) {
    let rechecked = |d: &Diagnostic| RECHECKED.contains(&d.code) && !d.fixes.is_empty();
    let found: Vec<Diagnostic> = diags.iter().filter(|d| rechecked(d)).cloned().collect();
    if found.is_empty() {
        return;
    }
    let before = check(src);
    let same = |check: &mut dyn FnMut(&str) -> Vec<(String, String)>, ds: &[Diagnostic]| {
        check(&crate::apply_fixes(src, ds).0) == before
    };
    if same(&mut check, &found) {
        return;
    }
    let wrong: Vec<_> = found
        .iter()
        .filter(|d| !same(&mut check, std::slice::from_ref(d)))
        .map(|d| d.span)
        .collect();
    diags.retain(|d| !(rechecked(d) && wrong.contains(&d.span)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recheck_reuses_the_modules_an_earlier_one_checked() {
        let src = "use std.graph.{Graph}\nlet g = Graph.undirected([(0, 1)])\nprint(g.n)\n";
        let lexed = || CHECKER.with_borrow(|db| db.stats().files_lexed);
        let mut diags = crate::lint(src, &crate::Config::default());
        recheck(&mut diags, src);
        let first = lexed();
        recheck(&mut diags, &src.replace("(0, 1)", "(1, 0)"));
        let again = lexed() - first;
        assert!(first > 2, "the first recheck checks std.graph: {first}");
        assert!(again <= 2, "only the file itself is checked again: {again}");
    }

    #[test]
    fn an_import_the_file_checks_differently_without_stays() {
        // The checker's verdict is a stand-in: `Rooted` gains no method
        // without its `use`, so dropping it is wrong; `Graph`'s is fine.
        let src = "use std.graph.{Graph}\nuse std.tree.{Rooted}\nlet g = Graph.undirected([(0, 1)])\nlet t = Rooted(g, 0)\nprint(t.size)\n";
        let mut diags = crate::lint(src, &crate::Config::default());
        let codes = |ds: &[Diagnostic]| {
            ds.iter()
                .filter(|d| d.code == REDUNDANT_PRELUDE_IMPORT.code)
                .count()
        };
        assert_eq!(codes(&diags), 2, "{diags:?}");
        keep_if_unchanged(&mut diags, src, |text| {
            if text.contains("use std.tree") {
                Vec::new()
            } else {
                vec![("E0203".into(), "`Rooted` has no method `size`".into())]
            }
        });
        assert_eq!(codes(&diags), 1);
        let left = diags
            .iter()
            .find(|d| d.code == REDUNDANT_PRELUDE_IMPORT.code)
            .unwrap();
        assert!(left.message.contains("`Graph`"), "{}", left.message);
    }

    #[test]
    fn an_import_the_file_checks_the_same_without_goes() {
        let src = "use std.graph.{Graph}\nlet g = Graph.undirected([(0, 1)])\nprint(g.n)\n";
        let mut diags = crate::lint(src, &crate::Config::default());
        let n = diags.len();
        recheck(&mut diags, src);
        assert_eq!(diags.len(), n);
        assert!(
            diags
                .iter()
                .any(|d| d.code == REDUNDANT_PRELUDE_IMPORT.code),
            "{diags:?}"
        );
    }
}
