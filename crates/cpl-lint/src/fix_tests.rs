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

pub(crate) fn checker_errors(src: &str) -> Vec<String> {
    let mut db = cpl_check::Session::new();
    db.set_file("probe.cp", src);
    let analysis = db.analyze("probe.cp").expect("analysis");
    cpl_ir::reported(&analysis)
        .iter()
        .filter(|d| d.severity == cpl_diag::Severity::Error)
        .map(|d| d.message.clone())
        .collect()
}

#[test]
fn a_hand_rolled_search_becomes_a_search_the_checker_knows() {
    // std/prefix.cp: the fix once wrote `bsearch_first`, which is not in the stdlib.
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

/// The parser's removed spellings and `++` are rewritten; nothing else the
/// parser offers to fix is.
#[test]
fn fix_removed_syntax_applies_removed_spellings_and_concatenation_only() {
    let (out, n) = fix_removed_syntax("let s = \"a\" ++ \"b\"\nprint(s)\n");
    assert_eq!((out.as_str(), n), ("let s = \"a\" + \"b\"\nprint(s)\n", 1));
    let (out, n) = fix_removed_syntax("let xs = [1, 2]\nfor x in xs where x > 1:\n    print(x)\n");
    assert_eq!(
        (out.as_str(), n),
        ("let xs = [1, 2]\nfor x in xs if x > 1:\n    print(x)\n", 1)
    );
}

/// A witness pins a plain name to its value; an expression has no input to pin.
#[test]
fn a_witness_names_only_a_plain_input() {
    assert_eq!(rules::witness("n_2", 5), vec![("n_2".to_string(), 5)]);
    for text in ["a[i]", "n + 1", "2n", ""] {
        assert!(rules::witness(text, 5).is_empty(), "{text}");
    }
}

#[test]
fn a_fix_that_would_add_a_checker_error_is_not_applied() {
    use cpl_diag::{Diagnostic, Span};
    let text = "let x = 1\nprint(x)\n";
    let at = |start, end| Span { start, end };
    let finding = |span, to: &str| {
        Diagnostic::warning(crate::LOOP_REDUCTION.code, span, "test").with_fix("fix", span, to)
    };
    let good = finding(at(8, 9), "2");
    let bad = finding(at(16, 17), "y");
    let mut db = cpl_check::Session::new();
    let (out, n) = crate::typed::safe_fixes(
        &mut db,
        "<t>.cp",
        text,
        &[bad.clone(), good.clone()],
        &[],
        &cpl_ir::check,
    );
    assert_eq!((out.as_str(), n), ("let x = 2\nprint(x)\n", 1));
    let (out, n) = crate::typed::safe_fixes(&mut db, "<t>.cp", text, &[bad], &[], &cpl_ir::check);
    assert_eq!((out.as_str(), n), (text, 0));
}

#[test]
fn a_fix_that_lowering_rejects_is_not_applied() {
    // noi15p3: an `s.any(..)` rewrite the checker took but lowering
    // rejected. The fix gate asks the complete check it is given.
    let text = "let x = 1\nprint(x)\n";
    let span = Span { start: 8, end: 9 };
    let finding =
        Diagnostic::warning(crate::LOOP_REDUCTION.code, span, "test").with_fix("fix", span, "2");
    let lowering_rejects_2 = move |a: &cpl_check::Analysis| {
        let mut out = a.diagnostics.clone();
        if a.text.contains("= 2") {
            out.push(Diagnostic::error(Code("E0302"), span, "lowering"));
        }
        out
    };
    let mut db = cpl_check::Session::new();
    let findings = [finding];
    let (out, n) =
        crate::typed::safe_fixes(&mut db, "<t>.cp", text, &findings, &[], &lowering_rejects_2);
    assert_eq!((out.as_str(), n), (text, 0));
    let (out, n) =
        crate::typed::safe_fixes(&mut db, "<t>.cp", text, &findings, &[], &cpl_ir::check);
    assert_eq!((out.as_str(), n), ("let x = 2\nprint(x)\n", 1));
}
