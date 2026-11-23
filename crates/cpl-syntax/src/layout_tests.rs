//! Recovery from indentation mistakes: one error each, with its fix.

use crate::*;

#[test]
fn an_indent_where_a_value_was_wanted_is_the_indent_error() {
    for src in [
        "@!multitest\n    let n: Int = read()\nprint(n)\n",
        "fn f(c: Char) -> Int:\n    match c:\n        'a' => 1\n            _ => 2\n",
    ] {
        let diags = parse_file(src).diagnostics;
        assert_eq!(diags.len(), 1, "{src:?}: {diags:#?}");
        let d = &diags[0];
        assert_eq!(d.code.0, "E0006", "{src:?}: {d:#?}");
        assert!(!d.fixes.is_empty(), "{src:?}: {d:#?}");
    }
}

#[test]
fn a_semicolon_after_a_decorator_is_one_error_with_a_removal() {
    let diags = parse_file("@!multitest;\nlet n: Int = read()\nprint(n)\n").diagnostics;
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(diags[0].message, "no `;` after a decorator", "{diags:?}");
    assert_eq!(diags[0].fixes[0].edits[0].1, "", "{diags:?}");
}

#[test]
fn a_leading_operator_continues_past_blank_and_comment_lines() {
    let src = "let b = a\n    // note\n    - 3\nlet c = xs\n    .min()\n\n    ?? 0\n";
    let parsed = parse_file(src);
    assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
    assert_eq!(parsed.items.len(), 2);
    let lexed = lexer::lex("let b = a\n    // note\n    - 3\n// after\nlet c = 1\n");
    assert_eq!(lexed.comments.len(), 2);
}
