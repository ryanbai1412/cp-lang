use crate::*;

fn kinds(src: &str) -> Vec<(TokenKind, &str)> {
    lex(src)
        .lossless(src)
        .iter()
        .map(|t| (t.kind, t.span.text(src)))
        .collect()
}

#[test]
fn trivia_fill_every_gap_between_tokens() {
    use TokenKind as T;
    let src = "fn f():  // c\n\tlet x = 1\r\n";
    let got = kinds(src);
    assert_eq!(got.iter().map(|t| t.1).collect::<String>(), src);
    assert!(got.contains(&(T::Comment, "// c")));
    assert!(got.contains(&(T::Whitespace, "  ")));
    assert!(got.contains(&(T::Whitespace, "\n\t")));
    assert!(got.contains(&(T::Whitespace, "\r")));
}

#[test]
fn a_skipped_header_is_one_trivia_token() {
    let src = "#include <bits/stdc++.h>\nprint(1)\n";
    let got = kinds(src);
    assert_eq!(got[0], (TokenKind::Skipped, "#include <bits/stdc++.h>"));
    assert_eq!(got.iter().map(|t| t.1).collect::<String>(), src);
}

#[test]
fn layout_tokens_come_before_the_trivia_at_their_offset() {
    let src = "if a:\n    b\n";
    let got = kinds(src);
    let newline = got.iter().position(|t| t.0 == TokenKind::Newline).unwrap();
    assert_eq!(got[newline + 1], (TokenKind::Whitespace, "\n    "));
}

#[test]
fn items_hold_their_tokens_and_the_file_holds_the_rest() {
    let src = "// head\nfn f():\n    pass\n\n\nlet x = 1 // tail\n";
    let file = parse(src);
    let items: Vec<String> = file.items().map(|i| i.text()).collect();
    assert_eq!(items.len(), 2);
    assert!(items[0].starts_with("// head\nfn f():"));
    assert!(items[1].starts_with("let x = 1"));
    assert_eq!(file.root.text(), src);
    let first = file.root.tokens().next().unwrap();
    assert_eq!(first.parent().kind(), NodeKind::Item);
}

#[test]
fn comment_lines_directly_above_an_item_are_in_it() {
    let src = "let a = 1

// loose

/// one
/// two
@inline
fn f():
    pass
";
    let file = parse(src);
    let f = file.items().nth(1).unwrap();
    assert!(f.text().starts_with("/// one\n/// two\n@inline\nfn f():"), "{:?}", f.text());
    let loose = file.root.tokens().find(|t| t.text() == "// loose").unwrap();
    assert_eq!(loose.parent().kind(), NodeKind::File);
    assert_eq!(file.root.text(), src);
}

#[test]
fn red_nodes_know_their_offsets_and_parents() {
    let src = "let a = 1\nlet b = 2\n";
    let file = parse(src);
    let second = file.items().nth(1).unwrap();
    assert_eq!(second.span().text(src).trim_end(), "let b = 2");
    assert_eq!(second.parent().unwrap().kind(), NodeKind::File);
    for t in file.root.tokens() {
        assert_eq!(t.span().text(src), t.text());
    }
}

#[test]
fn any_bytes_round_trip() {
    for src in ["", "\n", "(((", "fn f(:\n  x\n y\n", "\"unterminated", "'", "\t\t", "@", "a\rb", "é∑ // ü"] {
        assert_eq!(parse(src).root.text(), src, "{src:?}");
    }
}
