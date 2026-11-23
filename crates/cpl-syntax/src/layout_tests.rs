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

/// An inner call's `)` missing on one line of an outer call (noi08p5): the
/// next line is still an argument of the outer call, so one error.
#[test]
fn a_missing_close_paren_inside_a_multiline_call_is_one_error() {
    let src = "let d = f(\n    g(a, max(b, c),\n    g(0, c),\n)\nprint(d)\n";
    let file = parse_file(src);
    assert_eq!(file.diagnostics.len(), 1, "{:#?}", file.diagnostics);
    let d = &file.diagnostics[0];
    assert_eq!(d.code.0, "E0101");
    let at = src.find("c),\n    g(0").unwrap() + 2;
    assert_eq!(
        d.fixes[0].edits[0],
        (cpl_diag::Span::point(at as u32), ")".to_string())
    );
    assert_eq!(file.items.len(), 2);
}

/// A variant's `)` dropped before `| Next(..)` (noi08p5): after a `,` no
/// field starts with `|`, so the fix and its label go before it.
#[test]
fn a_missing_close_paren_before_an_infix_line_goes_before_it() {
    let src = "enum Op = Add(\n    t: Int,\n    c: Int,\n | Ask(t: Int)\nlet x = 1\n";
    let file = parse_file(src);
    assert_eq!(file.diagnostics.len(), 1, "{:#?}", file.diagnostics);
    let d = &file.diagnostics[0];
    let at = src.find("| Ask").unwrap() as u32;
    assert_eq!(
        d.fixes[0].edits[0],
        (cpl_diag::Span::point(at), ")".to_string())
    );
    assert_eq!(d.labels[0].span, cpl_diag::Span::point(at));
}
