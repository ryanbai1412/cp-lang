//! Recovery: one root cause, one diagnostic, and the rest still parses.

use crate::ast::*;
use crate::tests::{codes, one_diag, show, stmts};
use crate::*;

#[test]
fn a_missing_close_paren_is_one_error_and_the_rest_still_parses() {
    let src = "let x = foo(a, b\nlet y = x + 1\nprint(y)\n";
    let d = one_diag(src);
    assert_eq!(d.code.0, "E0101");
    assert_eq!(d.message, "this `(` is never closed");
    assert_eq!(d.span.text(src), "(");
    assert_eq!(
        d.fixes[0].edits[0],
        (cpl_diag::Span::point(16), ")".to_string())
    );
    let file = parse_file(src);
    assert_eq!(file.items.len(), 3);
    let StmtKind::Let {
        init: Some(init), ..
    } = &file.items[0].1.stmts[0].kind
    else {
        panic!()
    };
    assert_eq!(show(init), "foo(a, b)");
}

#[test]
fn a_dropped_closer_before_a_continued_line_goes_back_on_that_line() {
    for (src, rest) in [
        (
            "let xs = [3, 1].map(x => x + 1).zip(\n    [1, 2],\n.sorted()\nprint(xs)\n",
            ".sorted()",
        ),
        (
            "fn f(\n    a: Int,\n -> Int:\n    a\nprint(f(1))\n",
            "-> Int:",
        ),
    ] {
        let d = one_diag(src);
        assert_eq!(d.code.0, "E0101", "{src}");
        let at = src.find(rest).unwrap() as u32;
        assert_eq!(
            d.fixes[0].edits[0],
            (cpl_diag::Span::point(at), ")".to_string()),
            "{src}"
        );
        assert!(
            d.labels.iter().any(|l| l.span == cpl_diag::Span::point(at)),
            "{src}"
        );
    }
}

#[test]
fn nested_unclosed_brackets_are_still_one_error() {
    assert_eq!(codes("let x = foo(a[i, bar(b\nlet y = 1\n"), vec!["E0101"]);
    assert_eq!(codes("let x = foo(a, b"), vec!["E0101"]);
    assert_eq!(codes("let x = foo(a +\nlet y = 1\n"), vec!["E0101"]);
    assert_eq!(codes("print(xs[i)\n"), vec!["E0101"]);
}

#[test]
fn a_stray_closer_is_one_error() {
    let d = one_diag("let x = a + b)\nlet y = 1\n");
    assert_eq!(d.code.0, "E0109");
    assert_eq!(d.message, "this `)` has no matching opener");
}

#[test]
fn a_missing_value_is_one_error_at_the_end_of_the_line() {
    let src = "let x =\nlet y = x + 1\n";
    let d = one_diag(src);
    assert_eq!((d.code.0, d.span), ("E0102", cpl_diag::Span::point(7)));
    assert_eq!(
        d.message,
        "expected a value after `=`, found the end of the line"
    );
    assert_eq!(codes("let x = a +\n"), vec!["E0102"]);
    assert_eq!(codes("print(a, )\nlet z = 3 *\n"), vec!["E0102"]);
}

#[test]
fn a_half_typed_member_access_keeps_the_receiver() {
    let src = "let n = xs.\nprint(n)\n";
    let d = one_diag(src);
    assert_eq!(d.code.0, "E0108");
    let file = parse_file(src);
    let StmtKind::Let {
        init: Some(init), ..
    } = &file.items[0].1.stmts[0].kind
    else {
        panic!()
    };
    assert_eq!(show(init), "xs.");
}

#[test]
fn a_header_without_a_body_is_one_error() {
    let d = one_diag("for i in 0..n:\nprint(1)\n");
    assert_eq!(d.code.0, "E0104");
    assert_eq!(
        d.message,
        "expected an indented block after the `for` header"
    );
    assert_eq!(codes("fn f(x: Int) -> Int:\n"), vec!["E0104"]);
    assert_eq!(codes("if a > b:"), vec!["E0104"]);
    assert_eq!(codes("while true:\n\nlet x = 1\n"), vec!["E0104"]);
    // A nested header left unindented too stays inside the loop: one error,
    // and its body still sees `x`.
    let nested = "for x in a:\nif x > 10:\nprint(x)\n";
    assert_eq!(codes(nested), vec!["E0104"]);
    assert_eq!(parse_file(nested).items.len(), 1);
}

#[test]
fn a_missing_colon_is_one_error_and_the_body_is_still_parsed() {
    let src = "for i in 0..n\n    total += i\n    print(total)\nprint(1)\n";
    let d = one_diag(src);
    assert_eq!(d.code.0, "E0103");
    assert_eq!(
        d.fixes[0].edits[0],
        (cpl_diag::Span::point(13), ":".to_string())
    );
    let file = parse_file(src);
    let StmtKind::For { body, .. } = &file.items[0].1.stmts[0].kind else {
        panic!()
    };
    assert_eq!(body.stmts.len(), 2);
}

#[test]
fn errors_in_separate_statements_are_reported_separately() {
    let src = "fn f(x: Int) -> Int:\n    let a =\n    let b = x +\n    a + b\nlet y = f(\n";
    assert_eq!(codes(src), vec!["E0102", "E0102", "E0101"]);
}

#[test]
fn a_broken_header_does_not_hide_errors_in_its_body() {
    assert_eq!(
        codes("if x > :\n    let a =\n    print(a)\n"),
        vec!["E0102", "E0102"]
    );
}

#[test]
fn garbage_after_a_statement_is_one_error() {
    assert_eq!(
        codes("let x = 1 2 3 4\nlet y = 2;\n"),
        vec!["E0107", "E0107"]
    );
    assert_eq!(codes("print(a b c)\n"), vec!["E0103"]);
    assert_eq!(codes("else:\n    x\n"), vec!["E0107"]);
    assert_eq!(codes("let = 5\n"), vec!["E0106"]);
    assert_eq!(codes("let x: = 5\n"), vec!["E0105"]);
    assert_eq!(codes("fn (x: Int):\n    x\n"), vec!["E0108"]);
    assert_eq!(codes("struct P:\n    x Int\n    y: Int\n"), vec!["E0103"]);
    assert_eq!(codes("print(\"{a +}\")\n"), vec!["E0102"]);
}

#[test]
fn poisoned_spans_cover_what_the_parser_could_not_read() {
    let src = "let ok = 1\nlet x = foo(a b)\n";
    let file = parse_file(src);
    let (base, item) = &file.items[1];
    assert_eq!(item.poisoned.len(), 1);
    assert_eq!(item.poisoned[0].shift(*base).text(src), "let x = foo(a b)");
    assert!(file.items[0].1.poisoned.is_empty());
}

#[test]
fn the_parser_never_panics_or_hangs_on_truncated_input() {
    let src = "@memo\nfn solve(g: [V]List<(V, Int)>, src: V) -> [V]?Int:\n    var dist: [V]?Int = None\n    for (v, w) in g[src]:\n        if (dist[v] ?? INF) > w and not seen[v]:\n            dist[v] = w\n    match op:\n        Add(l, r) => st.add(l, r)\n        _ => print(\"x {l + 1}\")\n    dist\nlet a = [x * 2 for x in xs if x > 0]\nstruct P:\n    x, y: Int\n";
    for end in 0..=src.len() {
        if src.is_char_boundary(end) {
            let _ = parse_file(&src[..end]);
        }
    }
    // Every truncation point, with garbage appended.
    for end in (0..src.len()).step_by(7) {
        let _ = parse_file(&format!("{}) ] }} => : else", &src[..end]));
    }
}

#[test]
fn a_block_ends_its_expression_so_the_next_line_is_a_new_statement() {
    for (next, shown) in [("(a, b)", "Tuple"), ("[a, b]", "List"), ("-a", "Unary")] {
        let src = format!("if c:\n    return 1\n{next}\n");
        assert!(codes(&src).is_empty(), "{src}");
        let stmts = stmts(&src);
        assert_eq!(stmts.len(), 2, "{src}");
        let StmtKind::Expr(e) = &stmts[1].kind else {
            panic!("{src}")
        };
        assert!(
            format!("{:?}", e.kind).starts_with(shown),
            "{src}: {:?}",
            e.kind
        );
    }
    // On one line the operators still continue the expression.
    assert_eq!(stmts("x = (if c: 1 else: 2) - a\n").len(), 1);
}
