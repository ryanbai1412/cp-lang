use crate::ast::{self, AstNode};
use crate::*;
use crate::{SyntaxFile, SyntaxNode};

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
    assert!(
        f.text().starts_with("/// one\n/// two\n@inline\nfn f():"),
        "{:?}",
        f.text()
    );
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
    for src in [
        "",
        "\n",
        "(((",
        "fn f(:\n  x\n y\n",
        "\"unterminated",
        "'",
        "\t\t",
        "@",
        "a\rb",
        "é∑ // ü",
    ] {
        assert_eq!(parse(src).root.text(), src, "{src:?}");
    }
}

fn names(file: &SyntaxFile) -> Vec<Option<String>> {
    file.items()
        .filter_map(ast::Item::cast)
        .map(|i| i.name().map(|t| t.text().to_string()))
        .collect()
}

#[test]
fn items_name_their_declarations_and_impls_list_their_methods() {
    let src = "/// doc\nfn f(x: Int) -> Int:\n    x\nimpl Show for P:\n    fn show(self) -> Str = \"p\"\nimpl Q:\n    fn len(self) -> Int = 0\nlet y = 1\n";
    let file = parse(src);
    assert!(file.diagnostics.is_empty(), "{:?}", file.diagnostics);
    assert_eq!(names(&file), [Some("f".to_string()), None, None, None]);
    let impls: Vec<ast::ImplDecl> = file
        .items()
        .filter_map(|i| match ast::Item::cast(i)?.decl()? {
            ast::Decl::Impl(d) => Some(d),
            _ => None,
        })
        .collect();
    let text = |n: Option<SyntaxNode>| n.map(|n| n.text());
    assert_eq!(text(impls[0].trait_ref()).as_deref(), Some("Show"));
    assert_eq!(text(impls[0].self_ty()).as_deref(), Some("P"));
    assert_eq!(text(impls[1].trait_ref()), None);
    assert_eq!(text(impls[1].self_ty()).as_deref(), Some("Q"));
    let methods: Vec<String> = impls
        .iter()
        .filter_map(ast::ImplDecl::members)
        .flat_map(|m| m.fns().collect::<Vec<_>>())
        .filter_map(|f| f.name().map(|t| t.text().to_string()))
        .collect();
    assert_eq!(methods, ["show", "len"]);
    assert_eq!(file.root.text(), src);
}

#[test]
fn a_missing_name_reads_as_none_and_the_next_item_still_parses() {
    let src = "fn = 1\nstruct S:\n    a: Int\n";
    let file = parse(src);
    assert_eq!(file.diagnostics.len(), 2, "{:?}", file.diagnostics);
    assert_eq!(names(&file), [None, Some("S".to_string())]);
    assert_eq!(file.root.text(), src);
}

#[test]
fn consts_contexts_operator_fns_and_mod_items_are_named() {
    let src = "const N = 3\npub const PI: F64 = 3.1\npub context io:\n    base: Int = 0\nfn (a: P) + (b: P) -> P = a\nmod m:\n    fn f() = 1\n    struct S:\n        x: Int\nlet k = N\n";
    let file = parse(src);
    assert!(file.diagnostics.is_empty(), "{:?}", file.diagnostics);
    let want = [Some("N"), Some("PI"), Some("io"), None, Some("m"), None];
    assert_eq!(names(&file), want.map(|n| n.map(str::to_string)));
    let decls: Vec<ast::Decl> = file
        .items()
        .filter_map(|i| ast::Item::cast(i)?.decl())
        .collect();
    let ast::Decl::Fn(op) = &decls[3] else {
        panic!("not a fn: {decls:?}")
    };
    let op = op.op_name().and_then(|o| o.op());
    assert_eq!(op.map(|t| t.text().to_string()).as_deref(), Some("+"));
    let ast::Decl::Mod(m) = &decls[4] else {
        panic!("not a mod: {decls:?}")
    };
    let inner: Vec<_> = m
        .items()
        .map(|i| i.name().map(|t| t.text().to_string()))
        .collect();
    assert_eq!(inner, [Some("f".to_string()), Some("S".to_string())]);
    assert_eq!(file.root.text(), src);
}

#[test]
fn a_one_line_enum_after_a_colon_is_an_error_that_keeps_its_variants() {
    let src = "enum Dir: N, E\nlet x = 1\n";
    let file = parse(src);
    assert_eq!(file.diagnostics.len(), 1, "{:?}", file.diagnostics);
    let enum_ = file.items().next().unwrap();
    let variants = enum_
        .descendants()
        .filter(|n| n.kind() == NodeKind::Variant)
        .count();
    assert_eq!(variants, 2);
    assert_eq!(file.root.text(), src);
}

#[test]
fn a_decorator_on_the_item_line_heads_only_that_item() {
    let src = "@get pub fn r(self: R) -> Int = self.end\n@memo(map) fn f(x: D) -> Int = 7\nprint(f(D((1, 2))))\npub trait Signed\nimpl Signed for Int\nimpl Ord for P: fn lt(a, b) = b.v < a.v\ntype Z = Mod(p)\n";
    let file = parse(src);
    assert!(file.diagnostics.is_empty(), "{:?}", file.diagnostics);
    assert_eq!(file.items().count(), 7);
    assert_eq!(file.root.text(), src);
}
