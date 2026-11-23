//! `cpl fix` applies only fixes that edit inside the source.

use super::*;
use cpl_diag::{Code, Diagnostic, Span};

fn replacing(start: u32, end: u32, text: &str) -> Diagnostic {
    Diagnostic::error(Code("E0503"), Span::new(0, 1), "probe").with_fix(
        "probe",
        Span::new(start, end),
        text,
    )
}

#[test]
fn a_fix_outside_the_source_is_skipped() {
    let src = "let é = 1\n";
    let reversed = replacing(8, 2, "x");
    let past_the_end = replacing(4, 99, "x");
    let inside_a_char = replacing(5, 6, "x");
    let good = replacing(0, 3, "var");
    let (out, applied) = apply_fixes(src, &[reversed, past_the_end, inside_a_char, good]);
    assert_eq!((out.as_str(), applied), ("var é = 1\n", 1));
}

#[test]
fn a_reversed_else_if_fix_from_the_parser_is_left_alone() {
    // From corpus/problems/coci21c4p4/dream.cp: inside a closure the parser's
    // `else if` fix spans 142..133 (plan/domains/compiler/worklog.md).
    let src = "let xs = [1, 2]\nlet s = xs.fold(0, (acc, x) =>\n    var far = 0\n    for y in xs:\n        if y > 1:\n            far = 1\n        else if y > 0:\n            far = 2\n    far)\nprint(s)\n";
    let (out, _) = fix(src, &Config::default());
    assert!(out == src || out.contains("        elif y > 0:\n"), "{out}");
}

fn checker_errors(src: &str) -> Vec<String> {
    let mut db = cpl_check::Database::new();
    db.set_file("probe.cp", src);
    let analysis = db.analyze("probe.cp").expect("analysis");
    analysis
        .diagnostics
        .iter()
        .filter(|d| d.severity == cpl_diag::Severity::Error)
        .map(|d| d.message.clone())
        .collect()
}

#[test]
fn a_hand_rolled_search_becomes_a_search_the_checker_knows() {
    // spec/std/prefix.cp: the fix once wrote `bsearch_first`, which is not in the stdlib.
    let src = "fn f(a: List<Int>, k: Int) -> Int:\n    var lo = 0\n    var hi = a.len\n    while lo < hi:\n        let mid = (lo + hi) / 2\n        if a[mid] * a[mid] >= k: hi = mid\n        else: lo = mid + 1\n    lo\nfn g(ok: Int -> Bool, n: Int) -> Int:\n    var lo = 0\n    var hi = n - 1\n    while lo < hi:\n        let mid = lo + (hi - lo + 1) / 2\n        if ok(mid): lo = mid\n        else: hi = mid - 1\n    lo\nprint(f([1, 2, 3], 2), g(x => x < 3, 9))\n";
    assert!(checker_errors(src).is_empty());
    let (out, applied) = fix(src, &Config::default());
    assert_eq!(applied, 2, "{out}");
    assert!(
        out.contains(".first_true(") && out.contains(".last_true("),
        "{out}"
    );
    assert_eq!(checker_errors(&out), Vec::<String>::new(), "{out}");
}
