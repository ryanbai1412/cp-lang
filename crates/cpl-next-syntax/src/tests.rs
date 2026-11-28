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
    assert_eq!(
        text(impls[0].trait_ref().map(|n| n.syntax().clone())).as_deref(),
        Some("Show")
    );
    assert_eq!(
        text(impls[0].self_ty().map(|n| n.syntax().clone())).as_deref(),
        Some("P")
    );
    assert_eq!(text(impls[1].trait_ref().map(|n| n.syntax().clone())), None);
    assert_eq!(
        text(impls[1].self_ty().map(|n| n.syntax().clone())).as_deref(),
        Some("Q")
    );
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

#[test]
fn loop_is_reported_with_a_fix_and_parses_as_while() {
    let src = "loop:\n    break\n";
    let file = parse(src);
    assert_eq!(file.diagnostics.len(), 1, "{:?}", file.diagnostics);
    assert!(format!("{:?}", file.diagnostics[0]).contains("while true"));
    let stmt = file.items().next().unwrap();
    assert!(stmt.descendants().any(|n| n.kind() == NodeKind::WhileStmt));
    assert_eq!(file.root.text(), src);
}

fn decls(src: &str) -> Vec<ast::Decl> {
    let file = crate::parse(src);
    assert!(file.diagnostics.is_empty(), "{:#?}", file.diagnostics);
    assert_eq!(file.root.text(), src);
    file.root
        .children()
        .filter_map(ast::Item::cast)
        .filter_map(|i| i.decl())
        .collect()
}

fn texts(tokens: impl Iterator<Item = crate::tree::SyntaxToken>) -> Vec<String> {
    tokens.map(|t| t.text().to_string()).collect()
}

#[test]
fn a_fn_signature_reads_through_its_views() {
    let src = "@inline\npub fn gcd<T: Eq + Ord, const N: Int = 3>(a: T, by step: T; mut k = 1, var xs: List<T>, ..rest: Int) -> lends(a) mut T uses io where T: Copy = a\n";
    let [ast::Decl::Fn(f)] = &decls(src)[..] else {
        panic!()
    };
    assert!(f.is_pub());
    assert_eq!(texts(f.decorators().filter_map(|d| d.name())), ["inline"]);
    assert_eq!(f.name().unwrap().text(), "gcd");
    let g: Vec<_> = f.generics().unwrap().params().collect();
    assert_eq!(g[0].bound().unwrap().types().count(), 2);
    assert!(g[1].is_const() && g[1].ty().is_some());
    assert!(matches!(g[1].default(), Some(ast::Type::Literal(_))));
    let p: Vec<_> = f.params().unwrap().params().collect();
    let names: Vec<_> = p
        .iter()
        .map(|p| p.name().unwrap().text().to_string())
        .collect();
    assert_eq!(names, ["a", "step", "k", "xs", "rest"]);
    assert_eq!(p[1].label().unwrap().text(), "by");
    assert!(p[0].label().is_none() && !p[1].is_named_only());
    assert!(p[2].is_named_only() && p[2].is_mut() && p[2].default().is_some());
    assert!(p[3].is_var() && !p[3].is_mut() && p[4].is_variadic());
    let ret = f.ret().unwrap();
    assert!(ret.lends() && ret.lends_mut() && !ret.is_yields() && ret.ty().is_some());
    assert_eq!(texts(ret.lend_sources().unwrap().names()), ["a"]);
    assert_eq!(texts(f.uses().unwrap().contexts()), ["io"]);
    assert_eq!(f.where_clause().unwrap().preds().count(), 1);
    assert!(matches!(
        f.body(),
        Some(ast::FnBody::Expr(ast::Expr::Name(_)))
    ));
    let sig = f.signature_span();
    assert!(src[sig.start as usize..sig.end as usize].starts_with("pub fn gcd"));
    assert!(src[..sig.end as usize].ends_with("T: Copy"));
}

#[test]
fn members_fields_variants_and_impl_heads_read_through_their_views() {
    let src = "struct P(x: Int):\n    @cache\n    pub var y: ?Int = none\n    fn norm(self) -> Int: return self.x\nenum Op:\n    @tag(1)\n    Add(l: Int, r: Int)\n    Ask\ntrait Both = Eq + Ord\ntrait Seq(Eq):\n    type Elem\nimpl Seq for [V]Int:\n    type Elem = Int\nimpl P:\n    fn f(self) = 0\nuse lib.treap.{Treap, Node}\n";
    let d = decls(src);
    let ast::Decl::Struct(s) = &d[0] else {
        panic!()
    };
    let fields = s.fields();
    assert_eq!(fields.len(), 2);
    let y = &fields[1];
    assert_eq!(texts(y.decorators().filter_map(|d| d.name())), ["cache"]);
    assert!(y.is_pub() && y.is_var() && y.default().is_some());
    assert!(matches!(y.ty(), Some(ast::Type::Optional(_))));
    assert_eq!(s.members().unwrap().fns().count(), 1);
    let ast::Decl::Enum(e) = &d[1] else { panic!() };
    let v = e.variants();
    assert_eq!(texts(v.iter().filter_map(|v| v.name())), ["Add", "Ask"]);
    let tag = v[0].decorators().next().unwrap();
    assert_eq!(tag.name().unwrap().text(), "tag");
    assert_eq!(tag.args().unwrap().args().count(), 1);
    assert_eq!(v[0].fields().count(), 2);
    let ast::Decl::Trait(both) = &d[2] else {
        panic!()
    };
    assert!(both.alias().is_some_and(|b| b.types().count() == 2) && both.supertraits().is_none());
    let ast::Decl::Trait(seq) = &d[3] else {
        panic!()
    };
    assert_eq!(seq.supertraits().unwrap().types().count(), 1);
    let elem = seq.members().unwrap().assoc_types().next().unwrap();
    assert!(elem.name().is_some() && elem.ty().is_none());
    let ast::Decl::Impl(i) = &d[4] else { panic!() };
    assert!(i.trait_ref().is_some());
    let Some(ast::Type::Array(arr)) = i.self_ty() else {
        panic!()
    };
    assert_eq!(arr.dims().unwrap().args().count(), 1);
    assert!(matches!(arr.elem(), Some(ast::Type::Path(_))));
    assert!(i
        .members()
        .unwrap()
        .assoc_types()
        .next()
        .unwrap()
        .ty()
        .is_some());
    let ast::Decl::Impl(inherent) = &d[5] else {
        panic!()
    };
    assert!(inherent.trait_ref().is_none());
    let Some(ast::Type::Path(p)) = inherent.self_ty() else {
        panic!()
    };
    assert_eq!(texts(p.segments()), ["P"]);
    let ast::Decl::Use(u) = &d[6] else { panic!() };
    assert_eq!(texts(u.path()), ["lib", "treap"]);
    assert_eq!(texts(u.names().unwrap().names()), ["Treap", "Node"]);
}

#[test]
fn types_read_through_their_views() {
    let src = "fn g(f: (mut S, Int) -> Bool, r: 0..=n, s: Str of \"01\", t: Mod<7>, u: (lo: Int, ..R)) = 0\n";
    let [ast::Decl::Fn(g)] = &decls(src)[..] else {
        panic!()
    };
    let tys: Vec<_> = g
        .params()
        .unwrap()
        .params()
        .filter_map(|p| p.ty())
        .collect();
    let ast::Type::Fn(f) = &tys[0] else { panic!() };
    let Some(ast::Type::Tuple(args)) = f.param() else {
        panic!()
    };
    let muts: Vec<_> = args.elems().into_iter().map(|(m, _)| m).collect();
    assert_eq!(muts, [true, false]);
    assert!(matches!(f.ret(), Some(ast::Type::Path(_))));
    let ast::Type::Range(r) = &tys[1] else {
        panic!()
    };
    let Some(ast::Expr::Range(r)) = r.expr() else {
        panic!()
    };
    assert!(r.is_inclusive() && r.lo().is_some() && r.hi().is_some());
    let ast::Type::Alphabet(s) = &tys[2] else {
        panic!()
    };
    assert_eq!(s.alphabet().unwrap().text(), "\"01\"");
    let ast::Type::Path(t) = &tys[3] else {
        panic!()
    };
    let arg = t.generic_args().unwrap().args().next();
    assert!(matches!(
        arg,
        Some(ast::GenericArg::Type(ast::Type::Literal(_)))
    ));
    let ast::Type::Record(u) = &tys[4] else {
        panic!()
    };
    let rest: Vec<_> = u.fields().map(|f| f.is_rest()).collect();
    assert_eq!(rest, [false, true]);
}

#[test]
fn a_non_ascii_literal_names_its_literal_kind_not_a_type() {
    let message = |src| crate::lex(src).diagnostics[0].message.clone();
    assert!(message("print(\"naïve\")\n").contains("a string literal holds"));
    assert!(message("let c = 'é'\n").contains("a character literal holds"));
}

#[test]
fn only_the_for_keyword_starts_a_c_style_header() {
    let lexed = crate::lex("forest(x); y--\n");
    let want = cpl_diag::msgs::lexer::NO_DECREMENT.say(&[]);
    assert!(
        lexed.diagnostics.iter().any(|d| d.message == want),
        "{:?}",
        lexed.diagnostics
    );
}

#[test]
fn a_c_style_for_header_leaves_its_increment_to_the_parser() {
    let lexed = crate::lex("for (i = 0; i < n; i++)\n");
    let inc = cpl_diag::msgs::lexer::NO_INCREMENT.say(&[]);
    assert!(
        lexed.diagnostics.iter().all(|d| d.message != inc),
        "{:?}",
        lexed.diagnostics
    );
}
