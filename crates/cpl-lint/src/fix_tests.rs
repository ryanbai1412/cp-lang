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
