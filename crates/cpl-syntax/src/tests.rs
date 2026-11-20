use crate::ast::*;
use crate::token::TokenKind as T;
use crate::*;

fn kinds(src: &str) -> Vec<T> {
    lex(src).tokens.iter().map(|t| t.kind).collect()
}

/// Codes of every syntax diagnostic for `src`, in source order.
fn codes(src: &str) -> Vec<&'static str> {
    parse_file(src)
        .diagnostics
        .iter()
        .map(|d| d.code.0)
        .collect()
}

fn one_diag(src: &str) -> cpl_diag::Diagnostic {
    let diags = parse_file(src).diagnostics;
    assert_eq!(
        diags.len(),
        1,
        "expected exactly one diagnostic for {src:?}, got {diags:#?}"
    );
    diags.into_iter().next().unwrap()
}

fn stmts(src: &str) -> Vec<Stmt> {
    let file = parse_file(src);
    assert!(
        file.diagnostics.is_empty(),
        "unexpected diagnostics for {src:?}: {:#?}",
        file.diagnostics
    );
    file.items
        .into_iter()
        .flat_map(|(_, item)| item.stmts)
        .collect()
}

fn expr(src: &str) -> Expr {
    match stmts(src).pop().unwrap().kind {
        StmtKind::Expr(e) => e,
        other => panic!("not an expression: {other:?}"),
    }
}

/// Renders an expression with every grouping explicit.
fn show(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Int(v) => v.to_string(),
        ExprKind::Name(n) => n.clone(),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Unary { op, expr } => format!("({op:?} {})", show(expr)),
        ExprKind::Binary { op, lhs, rhs, .. } => {
            format!("({} {} {})", show(lhs), op.text(), show(rhs))
        }
        ExprKind::Compare { first, rest } => {
            let mut out = format!("(cmp {}", show(first));
            for (op, e) in rest {
                out += &format!(" {op:?} {}", show(e));
            }
            out + ")"
        }
        ExprKind::Range {
            lo,
            hi,
            inclusive,
            step,
        } => format!(
            "({}{}{}{})",
            lo.as_ref().map_or(String::new(), |e| show(e)),
            if *inclusive { "..=" } else { ".." },
            hi.as_ref().map_or(String::new(), |e| show(e)),
            step.as_ref()
                .map_or(String::new(), |e| format!(" by {}", show(e))),
        ),
        ExprKind::Call { callee, args } => format!(
            "{}({})",
            show(callee),
            args.iter()
                .map(|a| show(&a.value))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ExprKind::Index { base, indices } => {
            format!(
                "{}[{}]",
                show(base),
                indices.iter().map(show).collect::<Vec<_>>().join(", ")
            )
        }
        ExprKind::Field { base, name } => format!("{}.{}", show(base), name.text),
        ExprKind::Paren(e) => show(e),
        ExprKind::Generic { base, args } => format!("{}<{}>", show(base), args.len()),
        ExprKind::Missing => "<missing>".into(),
        other => format!("{other:?}"),
    }
}

// --- Lexer: layout -------------------------------------------------------

#[test]
fn indentation_becomes_indent_and_dedent_tokens() {
    let src = "if a:\n    b\n    if c:\n        d\ne\n";
    assert_eq!(
        kinds(src),
        vec![
            T::If,
            T::Ident,
            T::Colon,
            T::Newline,
            T::Indent,
            T::Ident,
            T::Newline,
            T::If,
            T::Ident,
            T::Colon,
            T::Newline,
            T::Indent,
            T::Ident,
            T::Newline,
            T::Dedent,
            T::Dedent,
            T::Ident,
            T::Newline,
            T::Eof,
        ]
    );
}

#[test]
fn blank_and_comment_lines_do_not_affect_layout() {
    let src = "if a:\n    b\n\n  // a comment at an odd indent\n    c\n";
    let lexed = lex(src);
    assert!(lexed.diagnostics.is_empty());
    assert_eq!(
        lexed.tokens.iter().filter(|t| t.kind == T::Indent).count(),
        1
    );
    assert_eq!(
        lexed.tokens.iter().filter(|t| t.kind == T::Dedent).count(),
        1
    );
    assert_eq!(lexed.comments.len(), 1);
}

#[test]
fn a_file_without_a_trailing_newline_still_closes_its_blocks() {
    assert_eq!(
        kinds("if a:\n    b"),
        vec![
            T::If,
            T::Ident,
            T::Colon,
            T::Newline,
            T::Indent,
            T::Ident,
            T::Newline,
            T::Dedent,
            T::Eof
        ]
    );
}

#[test]
fn newlines_inside_brackets_do_not_end_statements() {
    let src = "let x = f(\n    a,\n    b,\n)\nlet y = [\n1, 2\n]\n";
    let lexed = lex(src);
    assert!(lexed.diagnostics.is_empty(), "{:#?}", lexed.diagnostics);
    assert_eq!(
        lexed.tokens.iter().filter(|t| t.kind == T::Newline).count(),
        2
    );
    assert!(!lexed.tokens.iter().any(|t| t.kind == T::Indent));
}

#[test]
fn tabs_are_reported_once_per_file_with_a_fix_for_every_line() {
    let src = "if a:\n\tb\n\tc\n";
    let d = one_diag(src);
    assert_eq!(d.code.0, "E0001");
    assert_eq!(d.span, cpl_diag::Span::new(6, 7));
    assert_eq!(
        d.notes,
        vec!["1 more lines in this file are indented with tabs"]
    );
    assert_eq!(d.fixes[0].edits.len(), 2);
    // The block structure survives: the tab counts as an indent.
    assert_eq!(
        lex(src)
            .tokens
            .iter()
            .filter(|t| t.kind == T::Indent)
            .count(),
        1
    );
}

#[test]
fn a_dedent_to_no_enclosing_level_is_one_error() {
    let src = "if a:\n        b\n    c\nd\n";
    let d = one_diag(src);
    assert_eq!(d.code.0, "E0002");
    assert_eq!(
        d.message,
        "this line is indented 4 spaces, which matches no enclosing block (0)"
    );
    assert_eq!(d.fixes[0].edits[0].1, "");
}

#[test]
fn an_unexpected_indent_is_one_error_and_its_lines_are_kept() {
    let src = "let a = 1\n    let b = 2\n    let c = 3\nlet d = 4\n";
    let file = parse_file(src);
    assert_eq!(
        file.diagnostics
            .iter()
            .map(|d| d.code.0)
            .collect::<Vec<_>>(),
        vec!["E0006"]
    );
    let n: usize = file.items.iter().map(|(_, i)| i.stmts.len()).sum();
    assert_eq!(n, 4);
}

// --- Lexer: tokens -------------------------------------------------------

#[test]
fn numbers_ranges_and_floats() {
    assert_eq!(kinds("0..n")[..3], [T::Int, T::DotDot, T::Ident]);
    assert_eq!(kinds("1..=n")[..3], [T::Int, T::DotDotEq, T::Ident]);
    assert_eq!(
        kinds("1e9 2e5 1_000 0xff")[..4],
        [T::Int, T::Int, T::Int, T::Int]
    );
    assert_eq!(
        kinds("0.5 1.5e-9 1e-3")[..3],
        [T::Float, T::Float, T::Float]
    );
    assert_eq!(kinds("t.0")[..3], [T::Ident, T::Dot, T::Int]);
    assert_eq!(one_diag("let x = 12abc\n").code.0, "E0005");
}

#[test]
fn operators_lex_longest_first() {
    assert_eq!(
        kinds("a ** b **= c ?? d? -> => :: << <<= <= != ..=")[..],
        [
            T::Ident,
            T::StarStar,
            T::Ident,
            T::OpEq,
            T::Ident,
            T::QuestionQuestion,
            T::Ident,
            T::Question,
            T::Arrow,
            T::FatArrow,
            T::ColonColon,
            T::Shl,
            T::OpEq,
            T::LtEq,
            T::NotEq,
            T::DotDotEq,
            T::Newline,
            T::Eof
        ]
    );
}

#[test]
fn strings_chars_and_labels() {
    assert_eq!(
        kinds(r#""a {b("c")} d" r"\n" 'x' '\n' 'outer"#)[..5],
        [T::Str, T::RawStr, T::Char, T::Char, T::Label]
    );
    let d = one_diag("let s = \"abc\nlet t = 1\n");
    assert_eq!(d.code.0, "E0003");
    assert_eq!(d.fixes[0].edits[0].1, "\"");
}

#[test]
fn an_unknown_character_says_what_to_write_instead() {
    let d = one_diag("let a = b # c\n");
    assert_eq!(d.code.0, "E0004");
    assert_eq!(d.help.as_deref(), Some("comments start with `//`"));
}

#[test]
fn bang_unwraps_after_a_value_and_is_an_error_before_one() {
    assert!(matches!(
        expr("first_true(xs)!\n").kind,
        ExprKind::Unwrap(_)
    ));
    let d = one_diag("let a = !b\n");
    assert_eq!(d.code.0, "E0107");
    assert_eq!(d.fixes[0].edits[0].1, "not ");
}

#[test]
fn a_call_takes_a_generator_argument() {
    let ExprKind::Call { args, .. } = expr("max(dist[v] for v in V if seen[v])\n").kind else {
        panic!()
    };
    assert!(
        matches!(args[0].value.kind, ExprKind::Comprehension { ref clauses, .. } if clauses[0].cond.is_some())
    );
}

// --- Items ---------------------------------------------------------------

fn item_texts(src: &str) -> Vec<&str> {
    let lexed = lex(src);
    split_items(&lexed.tokens)
        .iter()
        .map(|i| i.span.text(src))
        .collect()
}

#[test]
fn items_are_top_level_statements_with_their_blocks() {
    let src = "let n = read()\n\nfn f(x: Int) -> Int:\n    let y = x\n    y + 1\n\nprint(f(n))\n";
    assert_eq!(
        item_texts(src),
        vec![
            "let n = read()",
            "fn f(x: Int) -> Int:\n    let y = x\n    y + 1",
            "print(f(n))"
        ]
    );
}

#[test]
fn decorators_and_else_branches_stay_with_their_item() {
    let src = "@memo\n@brute(slow)\nfn f(x: Int) -> Int:\n    x\nif a:\n    b\nelif c:\n    d\nelse:\n    e\nprint(1)\n";
    let items = item_texts(src);
    assert_eq!(items.len(), 3);
    assert!(items[0].starts_with("@memo") && items[0].ends_with("    x"));
    assert!(items[1].starts_with("if a") && items[1].ends_with("    e"));
    assert_eq!(items[2], "print(1)");
}

#[test]
fn item_trees_do_not_depend_on_where_the_item_is() {
    let body = "fn f(x: Int) -> Int:\n    let y = x * 2\n    y + 1\n";
    let a = parse_file(body);
    let b = parse_file(&format!("let pad = 1\n// moved down\n\n{body}"));
    assert_eq!(a.items[0].1, b.items[1].1);
    assert_eq!(b.items[1].0, 27);
}

// --- Parser: expressions --------------------------------------------------

#[test]
fn precedence() {
    assert_eq!(show(&expr("a + b * c\n")), "(a + (b * c))");
    assert_eq!(show(&expr("-x ** 2\n")), "(Neg (x ** 2))");
    assert_eq!(show(&expr("2 ** 3 ** 2\n")), "(2 ** (3 ** 2))");
    assert_eq!(show(&expr("a or b and not c\n")), "(a or (b and (Not c)))");
    assert_eq!(show(&expr("a & 1 == 0\n")), "(cmp (a & 1) Eq 0)");
    assert_eq!(show(&expr("lo <= x < hi\n")), "(cmp lo Le x Lt hi)");
    assert_eq!(show(&expr("d[v] ?? inf > w\n")), "(d[v] ?? (cmp inf Gt w))");
    assert_eq!(show(&expr("0..n - 1 by 2\n")), "(0..(n - 1) by 2)");
    assert_eq!(show(&expr("x not in seen\n")), "(cmp x NotIn seen)");
    assert_eq!(show(&expr("g[r, c].len\n")), "g[r, c].len");
}

#[test]
fn capitalized_name_with_closed_type_arguments_is_generic() {
    assert_eq!(show(&expr("SegTree<V, Int>(sum)\n")), "SegTree<2>(sum)");
    assert_eq!(show(&expr("Map<Int, List<Int>>()\n")), "Map<2>()");
    assert_eq!(show(&expr("INF < x\n")), "(cmp INF Lt x)");
    assert_eq!(
        show(&expr("A < B and C > d\n")),
        "((cmp A Lt B) and (cmp C Gt d))"
    );
    assert_eq!(show(&expr("f::<Int>(x)\n")), "f<1>(x)");
}

#[test]
fn lambdas_tables_and_comprehensions() {
    assert!(
        matches!(expr("x => x + 1\n").kind, ExprKind::Lambda { ref params, .. } if params.len() == 1)
    );
    assert!(
        matches!(expr("(a, b) => a + b\n").kind, ExprKind::Lambda { ref params, .. } if params.len() == 2)
    );
    assert!(
        matches!(expr("[V] u => deg[u] * 2\n").kind, ExprKind::Table { ref domains, .. } if domains.len() == 1)
    );
    assert!(
        matches!(expr("[deg[u] * 2 for u in V if deg[u] > 0]\n").kind, ExprKind::Comprehension { ref clauses, .. } if clauses[0].cond.is_some())
    );
    let nested = expr("[(i, j) for i in I for j in I if i < j]\n");
    assert!(
        matches!(nested.kind, ExprKind::Comprehension { ref clauses, .. } if clauses.len() == 2)
    );
    assert!(matches!(expr("[1, 2, 3]\n").kind, ExprKind::List(ref xs) if xs.len() == 3));
    let call = expr("sort(xs, by: a => { let k = key(a); (k.x, -k.y) })\n");
    let ExprKind::Call { args, .. } = call.kind else {
        panic!()
    };
    assert_eq!(args[1].name.as_ref().unwrap().text, "by");
    let ExprKind::Lambda { body, .. } = &args[1].value.kind else {
        panic!()
    };
    assert!(matches!(&body.kind, ExprKind::Block(b) if b.stmts.len() == 2));
}

#[test]
fn calls_take_named_and_mut_arguments() {
    let ExprKind::Call { args, .. } = expr("link(mut g, r, weight: 3)\n").kind else {
        panic!()
    };
    assert!(args[0].mutable && !args[1].mutable);
    assert_eq!(args[2].name.as_ref().unwrap().text, "weight");
}

#[test]
fn strings_interpolate_with_spans_into_the_string() {
    let src = "print(\"Case #{t}: {ans + 1}\")\n";
    let ExprKind::Call { args, .. } = expr(src).kind else {
        panic!()
    };
    let ExprKind::Str(pieces) = &args[0].value.kind else {
        panic!()
    };
    assert_eq!(pieces.len(), 4);
    let StrPiece::Expr(t) = &pieces[1] else {
        panic!()
    };
    assert_eq!(t.span.text(src), "t");
    let StrPiece::Expr(sum) = &pieces[3] else {
        panic!()
    };
    assert_eq!(sum.span.text(src), "ans + 1");
    assert_eq!(pieces[2], StrPiece::Text(": ".into()));
}

#[test]
fn inline_and_block_conditionals() {
    let ExprKind::If {
        branches,
        otherwise,
    } = expr("if a < b: b - a else: a - b\n").kind
    else {
        panic!()
    };
    assert_eq!(branches.len(), 1);
    assert!(otherwise.is_some());
    let s = stmts("if a:\n    x = 1\nelif b:\n    x = 2\nelse:\n    x = 3\n");
    let StmtKind::Expr(Expr {
        kind: ExprKind::If {
            branches,
            otherwise,
        },
        ..
    }) = &s[0].kind
    else {
        panic!()
    };
    assert_eq!(branches.len(), 2);
    assert!(otherwise.is_some());
    let s = stmts("let d = if a < b: b - a else: a - b\nif let v = find(x): print(v)\n");
    assert_eq!(s.len(), 2);
}

// --- Parser: statements and declarations -----------------------------------

#[test]
fn bindings_assignments_and_loops() {
    let s = stmts("let n: 1..=2e5 = read()\nvar best = 0\nbest max= n\nbest += 1\nfor i in 0..n:\n    best = i\nwhile best > 0:\n    best /= 2\n");
    assert!(matches!(
        &s[0].kind,
        StmtKind::Let {
            mutable: false,
            ty: Some(Type {
                kind: TypeKind::Range {
                    inclusive: true,
                    ..
                },
                ..
            }),
            ..
        }
    ));
    assert!(matches!(&s[1].kind, StmtKind::Let { mutable: true, .. }));
    assert!(matches!(
        &s[2].kind,
        StmtKind::Assign {
            op: AssignOp::Max,
            ..
        }
    ));
    assert!(matches!(
        &s[3].kind,
        StmtKind::Assign {
            op: AssignOp::Binary(BinOp::Add),
            ..
        }
    ));
    assert!(matches!(&s[4].kind, StmtKind::For { .. }));
    assert!(matches!(&s[5].kind, StmtKind::While { .. }));
}

#[test]
fn fn_forms() {
    let s = stmts("fn dist(a: P, b: P) -> Int:\n    abs(a.x - b.x)\nfn sq(x: Int) -> Int = x * x\nfn link(mut g: Dsu, r: Road; weight: Int = 1, ..rest: Int):\n    g.join(r.u, r.v)\nfn peel<D>(s: [D]Bool) -> ?D:\n    none\n");
    let StmtKind::Fn(f) = &s[0].kind else {
        panic!()
    };
    assert_eq!((f.name.text.as_str(), f.params.len()), ("dist", 2));
    assert!(matches!(f.body, FnBody::Block(_)));
    let StmtKind::Fn(f) = &s[1].kind else {
        panic!()
    };
    assert!(matches!(f.body, FnBody::Expr(_)));
    let StmtKind::Fn(f) = &s[2].kind else {
        panic!()
    };
    assert!(f.params[0].mutable && !f.params[1].named_only);
    assert!(f.params[2].named_only && f.params[2].default.is_some());
    assert!(f.params[3].variadic);
    let StmtKind::Fn(f) = &s[3].kind else {
        panic!()
    };
    assert_eq!(f.generics[0].text, "D");
    assert!(matches!(
        f.ret.as_ref().unwrap().kind,
        TypeKind::Optional(_)
    ));
}

#[test]
fn types() {
    let s = stmts("let a: [V]?0..=1e9 = read()\nlet g: [Row, Col]Char = read()\nlet s: [domain S]Char = read()\nlet m: Map<Int, List<(V, Int)>> = Map()\nlet r: (lo: Int, hi: V) = f()\n");
    let ty = |i: usize| match &s[i].kind {
        StmtKind::Let { ty: Some(t), .. } => t.kind.clone(),
        _ => panic!(),
    };
    assert!(
        matches!(ty(0), TypeKind::Array { ref index, ref elem } if index.len() == 1 && matches!(elem.kind, TypeKind::Optional(_)))
    );
    assert!(matches!(ty(1), TypeKind::Array { ref index, .. } if index.len() == 2));
    assert!(matches!(ty(2), TypeKind::OwnDomain { .. }));
    assert!(matches!(ty(3), TypeKind::Named { ref args, .. } if args.len() == 2));
    assert!(matches!(ty(4), TypeKind::Record(ref f) if f.len() == 2));
}

#[test]
fn counted_arrays_derived_domains_generators_and_do_blocks() {
    let s = stmts("let xs: [q]1..=1e9 = read()\nvar pre: [V.bounds]Int = 0\nfn divisors(n: 1..=1000) -> yields Int:\n    yield 1\nlet ans: ?Int = do:\n    let x = find(xs)?\n    x + 1\n");
    let StmtKind::Let {
        ty: Some(Type {
            kind: TypeKind::Array { index, .. },
            ..
        }),
        ..
    } = &s[0].kind
    else {
        panic!()
    };
    assert!(matches!(
        index[0].kind,
        TypeKind::Range {
            inclusive: false,
            ..
        }
    ));
    let StmtKind::Let {
        ty: Some(Type {
            kind: TypeKind::Array { index, .. },
            ..
        }),
        ..
    } = &s[1].kind
    else {
        panic!()
    };
    assert!(matches!(&index[0].kind, TypeKind::Named { name, .. } if name.text == "V.bounds"));
    let StmtKind::Fn(f) = &s[2].kind else {
        panic!()
    };
    assert!(matches!(f.ret.as_ref().unwrap().kind, TypeKind::Yields(_)));
    assert!(
        matches!(&s[3].kind, StmtKind::Let { init: Some(Expr { kind: ExprKind::Block(b), .. }), .. } if b.stmts.len() == 2)
    );
}

#[test]
fn structs_enums_domains_and_match() {
    let s = stmts("domain V = 1..=n\nstruct Road:\n    u, v: V\n    len: 1..=1e9 = 1\n    fn other(self, x: V) -> V:\n        if x == self.u: self.v else: self.u\nenum Dir = L | R | U | D\nenum Op:\n    Add(l: V, r: V)\n    Ask(V)\nmatch op:\n    Add(l, r) => st.add(l, r)\n    Ask(i) =>\n        let x = st.get(i)\n        print(x)\n    _ => print(0)\nuse std.segtree\nuse lib.treap.{Treap, Node}\nuse lib.graph.flow as fl\n");
    assert!(matches!(&s[0].kind, StmtKind::Domain { name, .. } if name.text == "V"));
    let StmtKind::Struct(road) = &s[1].kind else {
        panic!()
    };
    assert_eq!(
        road.fields
            .iter()
            .map(|f| f.name.text.as_str())
            .collect::<Vec<_>>(),
        ["u", "v", "len"]
    );
    assert_eq!(road.methods.len(), 1);
    let StmtKind::Enum(dir) = &s[2].kind else {
        panic!()
    };
    assert_eq!(dir.variants.len(), 4);
    let StmtKind::Enum(op) = &s[3].kind else {
        panic!()
    };
    assert_eq!(op.variants[0].fields.len(), 2);
    let StmtKind::Expr(Expr {
        kind: ExprKind::Match { arms, .. },
        ..
    }) = &s[4].kind
    else {
        panic!()
    };
    assert_eq!(arms.len(), 3);
    assert_eq!(arms[1].body.stmts.len(), 2);
    assert!(matches!(&s[6].kind, StmtKind::Use { names, .. } if names.len() == 2));
    assert!(matches!(&s[7].kind, StmtKind::Use { alias: Some(a), .. } if a.text == "fl"));
}

#[test]
fn traits_may_leave_bodies_out() {
    let s = stmts("trait Index:\n    const K: Int\n    fn to_idx(self) -> 0..K\n    fn from_idx(i: 0..K) -> Self\n");
    let StmtKind::Trait { body, .. } = &s[0].kind else {
        panic!()
    };
    assert_eq!(body.stmts.len(), 3);
}

// --- Parser: one root cause, one diagnostic --------------------------------

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
    let src = "@memo\nfn solve(g: [V]List<(V, Int)>, src: V) -> [V]?Int:\n    var dist: [V]?Int = none\n    for (v, w) in g[src]:\n        if (dist[v] ?? INF) > w and not seen[v]:\n            dist[v] = w\n    match op:\n        Add(l, r) => st.add(l, r)\n        _ => print(\"x {l + 1}\")\n    dist\nlet a = [x * 2 for x in xs if x > 0]\nstruct P:\n    x, y: Int\n";
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
