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
        matches!(args[0].value.kind, ExprKind::Comprehension { ref clauses, .. } if clauses[0].conds.len() == 1)
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
    assert_eq!(show(&expr("read<(Int, Int)>()\n")), "read<1>()");
    assert_eq!(show(&expr("read<Int>()\n")), "read<1>()");
    assert_eq!(
        show(&expr("i < n and m > (k)\n")),
        "((cmp i Lt n) and (cmp m Gt k))"
    );
    assert!(!show(&expr("a < b > (c)\n")).contains("a<"));
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
        matches!(expr("[deg[u] * 2 for u in V if deg[u] > 0]\n").kind, ExprKind::Comprehension { ref clauses, .. } if clauses[0].conds.len() == 1)
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
    let lz = stmts("fn f(lazy d: () -> Int, lazy: Int) -> Int = lazy\n");
    let StmtKind::Fn(f) = &lz[0].kind else {
        panic!()
    };
    assert!(f.params[0].lazy && !f.params[1].lazy && f.params[1].name.text == "lazy");
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
    let s = stmts("let xs: [q]1..=1e9 = read()\nvar pre: [V.cuts]Int = 0\nfn divisors(n: 1..=1000) -> yields Int:\n    yield 1\nlet ans: ?Int = do:\n    let x = find(xs)?\n    x + 1\n");
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
    assert!(matches!(&index[0].kind, TypeKind::Named { name, .. } if name.text == "V.cuts"));
    let StmtKind::Fn(f) = &s[2].kind else {
        panic!()
    };
    assert!(matches!(f.ret.as_ref().unwrap().kind, TypeKind::Yields(_)));
    assert!(
        matches!(&s[3].kind, StmtKind::Let { init: Some(Expr { kind: ExprKind::Block(b), .. }), .. } if b.stmts.len() == 2)
    );
}

#[test]
fn grader_domain_without_range() {
    let s = stmts("domain Row\ndomain Col = 0..m\n");
    assert!(matches!(&s[0].kind, StmtKind::Domain { name, .. } if name.text == "Row"));
    assert!(matches!(&s[1].kind, StmtKind::Domain { name, .. } if name.text == "Col"));
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

#[test]
fn an_unclosed_paren_fix_lands_where_the_closer_belongs() {
    // The next line is no deeper: close at the end of the opener's line, not
    // before a later `=`.
    let src = "var a = [1]\nif true:\n    a[0] = max(a[0], 0\n    if a[0] > 0:\n        a[0] = 1\nprint(a)\n";
    let d = one_diag(src);
    assert_eq!(d.code.0, "E0101");
    let at = src.find(", 0").unwrap() + 3;
    assert_eq!(
        d.fixes[0].edits[0],
        (cpl_diag::Span::point(at as u32), ")".to_string())
    );
    // No argument starts with `:`, `!` or `>`: the call was empty.
    for src in [
        "for x in read<List<Int>>(:\n    print(x)\n",
        "let x = xs.pop(!\nprint(x)\n",
    ] {
        let d = one_diag(src);
        let at = src.find('(').unwrap() + 1;
        assert_eq!(
            d.fixes[0].edits[0].0,
            cpl_diag::Span::point(at as u32),
            "{src}"
        );
    }
    // Before `in` after `for (`, before `=>` on a line's opener, and
    // between a literal and a name.
    for (src, close_after) in [
        ("for (i, x in [1].items:\n    print(i)\n", "x"),
        ("enum E:\n    @tag(\"3\" Stop\n", "\"3\""),
        (
            "let v = match (3, 1):\n    (3, 1 => 1\n    _ => 0\nprint(v)\n",
            "(3, 1",
        ),
    ] {
        let d = one_diag(src);
        let at = src.rfind(close_after).unwrap() + close_after.len();
        let insert = (cpl_diag::Span::point(at as u32), ")".to_string());
        assert_eq!(d.fixes[0].edits[0], insert, "{src}");
    }
}

#[test]
fn a_condition_with_a_bare_equals_gets_the_comparison_fix() {
    let src = "let n = 1\nif n = 1:\n    print(n)\n";
    let d = one_diag(src);
    assert_eq!(d.code.0, "E0103");
    let at = src.find(" = 1:").unwrap() + 1;
    let eq = cpl_diag::Span {
        start: at as u32,
        end: at as u32 + 1,
    };
    assert_eq!(d.fixes[0].edits[0], (eq, "==".to_string()));
    // The line already ends in `:`, so inserting another is no fix.
    let d = one_diag("let n = 1\nwhile n n:\n    print(n)\n");
    assert!(d.fixes.is_empty(), "{:?}", d.fixes);
    // `==` would leave `n == *= 1`, still broken.
    let d = one_diag("let n = 1\nif n = *= 1:\n    print(n)\n");
    assert!(d.fixes.is_empty(), "{:?}", d.fixes);
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

// --- Library syntax: generics, traits, projections --------------------

fn one_stmt(src: &str) -> Stmt {
    let mut all = stmts(src);
    assert_eq!(all.len(), 1, "expected one statement for {src:?}");
    all.pop().unwrap()
}

fn fn_of(src: &str) -> FnDecl {
    match one_stmt(src).kind {
        StmtKind::Fn(f) => f,
        other => panic!("not a fn: {other:?}"),
    }
}

fn struct_of(src: &str) -> StructDecl {
    match one_stmt(src).kind {
        StmtKind::Struct(s) => s,
        other => panic!("not a struct: {other:?}"),
    }
}

#[test]
fn an_expression_body_may_sit_on_the_next_lines() {
    let f =
        fn_of("fn count_brute() -> Int =\n    sum(1 for l in I\n        for r in I if l <= r)\n");
    assert!(matches!(
        f.body,
        FnBody::Expr(Expr {
            kind: ExprKind::Call { .. },
            ..
        })
    ));
    // The fn ends with its block; the next statement is separate.
    assert_eq!(stmts("fn f() -> Int =\n    1 + 2\nprint(f())\n").len(), 2);
    // `=` takes one expression; statements below it say to use `:`.
    let diag = one_diag("fn f() -> Int =\n    let x = 1\n    x\n");
    assert!(
        diag.message.contains("starts with `:`, not `=`") && diag.fixes.len() == 1,
        "{}",
        diag.message
    );
    let diag = one_diag("fn f() -> Int =\n    1\n    2\n");
    assert!(
        diag.message.contains("is one expression"),
        "{}",
        diag.message
    );
}

#[test]
fn type_parameters_carry_their_bounds() {
    let f = fn_of("fn best<T: Ord + Bounded, const N: Int>(xs: List<T>) -> T: xs[0]\n");
    let names: Vec<&str> = f.generics.iter().map(|g| g.text.as_str()).collect();
    assert_eq!(names, ["T", "N"]);
    let bounds: Vec<(&str, usize)> = f
        .bounds
        .iter()
        .map(|b| (b.name.text.as_str(), b.traits.len()))
        .collect();
    assert_eq!(bounds, [("T", 2)]);
    assert!(matches!(
        one_stmt("pub const Max<T: Ord + Bounded> = Monoid<T>(id: T.MIN, op: (a, b) => max(a, b))\n"),
        Stmt { public: true, kind: StmtKind::Const { ref generics, ref bounds, .. }, .. }
            if generics.len() == 1 && bounds[0].traits.len() == 2
    ));
}

#[test]
fn fn_types_value_arguments_and_owned_domains() {
    let f = fn_of("fn bfs<V, A>(adj: V -> Iter<A>, from: List<V>) -> (domain C, comp: [V]C): x\n");
    assert!(matches!(
        f.params[0].ty.as_ref().unwrap().kind,
        TypeKind::Fn { ref params, .. } if params.len() == 1
    ));
    assert!(matches!(
        f.ret.as_ref().unwrap().kind,
        TypeKind::Record(ref fields)
            if fields[0].0.text == "C" && fields[0].1.kind == TypeKind::NewDomain && fields.len() == 2
    ));
    assert!(matches!(
        one_stmt("type M = Mod<998244353>\n").kind,
        StmtKind::TypeAlias { ty: Type { kind: TypeKind::Named { ref args, .. }, .. }, .. }
            if matches!(args[0].kind, TypeKind::Value(_))
    ));
    assert!(matches!(
        one_stmt("let down: SegTree<Rooted<V>.Pre, T> = x\n").kind,
        StmtKind::Let { ty: Some(Type { kind: TypeKind::Named { ref args, .. }, .. }), .. }
            if matches!(args[0].kind, TypeKind::Named { ref name, ref args } if name.text == "Rooted.Pre" && args.len() == 1)
    ));
}

#[test]
fn type_arguments_close_before_a_comma_or_a_closer() {
    for src in [
        "laws_hold(Sum<Int>, xs)\n",
        "fold(xs, Max<Int>)\n",
        "let ms = [Mat<7>.of(a), Mat<7>.identity(2)]\n",
        "sort(mut xs, by: Greater<(Int, Int)>)\n",
        "let t = Fenwick<V, Mod<P>>(g)\n",
    ] {
        stmts(src);
    }
    // Comparisons between constants are still comparisons.
    assert_eq!(
        show(&expr("f(N < M, K > 2)\n")),
        "f((cmp N Lt M), (cmp K Gt 2))"
    );
    assert!(matches!(
        expr("[A < B, C]\n").kind,
        ExprKind::List(ref items) if matches!(items[0].kind, ExprKind::Compare { .. })
    ));
}

#[test]
fn structs_take_fields_in_the_header_and_declarations_in_the_body() {
    let s = struct_of("pub struct Monoid<T>(id: T, op: (T, T) -> T)\n");
    assert_eq!((s.fields.len(), s.methods.len()), (2, 0));
    let s = struct_of(
        "struct Fenwick<D, T: Add>(g: Group<T>):\n    t: List<T>\n    domain Pre = 0..D.K\n    @law assoc(a: T, b: T) = g.op(a, b) == g.op(b, a)\n    pub @get fn count(self) -> Int = t.len\n",
    );
    let fields: Vec<&str> = s.fields.iter().map(|f| f.name.text.as_str()).collect();
    assert_eq!(fields, ["g", "t"]);
    assert_eq!(s.bounds.len(), 1);
    assert!(matches!(s.methods[0].kind, StmtKind::Domain { .. }));
    assert!(matches!(s.methods[1].kind, StmtKind::Law { ref params, .. } if params.len() == 2));
    assert!(matches!(
        s.methods[2],
        Stmt { public: true, kind: StmtKind::Fn(_), ref decorators, .. } if decorators[0].name.text == "get"
    ));
    assert!(matches!(
        one_stmt("enum Op:\n    Merge(a, b: Ship)\n    Ask(Int)\n").kind,
        StmtKind::Enum(EnumDecl { ref variants, .. })
            if variants[0].fields.len() == 2 && variants[0].fields[0].0.is_some() && variants[1].fields[0].0.is_none()
    ));
}

#[test]
fn traits_and_impls_have_supertraits_bounds_and_optional_bodies() {
    assert!(matches!(
        one_stmt("trait Seq: Index + Ord:\n    type bounds = Cut<Self>\n    type Elem\n    fn next(self) -> ?Self\n").kind,
        StmtKind::Trait { ref supers, ref body, .. } if supers.len() == 2 && body.stmts.len() == 3
    ));
    assert!(matches!(
        one_stmt("impl Seq for V\n").kind,
        StmtKind::Impl { trait_name: Some(_), ref body, .. } if body.stmts.is_empty()
    ));
    assert!(matches!(
        one_stmt("impl<I: Seq> Indexed<Range<I>> for Array<I, T>:\n    let K = n\n    fn add(a, b) = a\n").kind,
        StmtKind::Impl { ref bounds, ref body, .. } if bounds.len() == 1 && body.stmts.len() == 2
    ));
    assert!(matches!(
        one_stmt("impl Show for V: fn show(self) = show(self.label)\n").kind,
        StmtKind::Impl { ref body, .. } if body.stmts.len() == 1
    ));
}

#[test]
fn projections_lend_and_state_laws() {
    let s = struct_of(
        "struct Bag:\n    cnt: [V]Int\n    fn at(self, i: V) lends mut Int:\n        @law disjoint(i, j) = i != j\n        mut cnt[i]\n    fn peek(self, i: V) lends Int:\n        cnt[i]\n",
    );
    let StmtKind::Fn(at) = &s.methods[0].kind else {
        panic!()
    };
    assert!(at.lends.as_ref().is_some_and(|l| l.mutable));
    let FnBody::Block(body) = &at.body else {
        panic!()
    };
    assert!(
        matches!(body.stmts[0].kind, StmtKind::Law { ref name, .. } if name.text == "disjoint")
    );
    assert!(matches!(
        body.stmts[1].kind,
        StmtKind::Lend { mutable: true, .. }
    ));
    let StmtKind::Fn(peek) = &s.methods[1].kind else {
        panic!()
    };
    assert!(peek.lends.as_ref().is_some_and(|l| !l.mutable));
    // `lend` and `law` are ordinary names in v2.2.
    stmts("let lend = 1\nlet law = lend + 1\nlend = 2\nprint(law)\n");
}

#[test]
fn bindings_take_else_branches_and_refinements() {
    assert!(matches!(
        one_stmt("let v = u else:\n    return\n").kind,
        StmtKind::Let {
            otherwise: Some(_),
            guard: None,
            ..
        }
    ));
    // v2.2 has no `where`: the constraint goes in the type or an `assert`.
    let diag = one_diag("let k: 0..n = read(gen: fewer) where k < n\n");
    assert_eq!(diag.code.0, "E0503");
    assert!(diag.help.is_some());
    assert!(matches!(
        expr("max(price[s] - price[b] for c in City if let b = buy[c] if let s = sell[c] if b < s)\n").kind,
        ExprKind::Call { ref args, .. }
            if matches!(args[0].value.kind, ExprKind::Comprehension { ref clauses, .. } if clauses[0].conds.len() == 3)
    ));
}

#[test]
fn by_is_a_name_except_after_a_range() {
    stmts("fn shift(c: Letter, by: Int):\n    have[c] += by\n");
    stmts("let (bx, by) = (1, 2)\nprint(by)\n");
    stmts("struct Order<T>(lt: (T, T) -> Bool):\n    pub fn by(key: T -> Int) -> Order<T> = Order((a, b) => key(a) < key(b))\n");
    assert!(matches!(
        expr("0..n by 2\n").kind,
        ExprKind::Range { step: Some(_), .. }
    ));
}

#[test]
fn any_fn_of_two_values_updates_in_place() {
    // `l lcm= x` is `l = lcm(l, x)`.
    let StmtKind::Assign { target, op, value } = one_stmt("each[i] lcm= a[i] * 2\n").kind else {
        panic!("not an assignment")
    };
    assert_eq!(op, AssignOp::Set);
    let ExprKind::Call { callee, args } = value.kind else {
        panic!("not a call")
    };
    assert_eq!(callee.kind, ExprKind::Name("lcm".into()));
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].value, target);
    // `max=` and `min=` keep their own operators.
    assert!(matches!(
        one_stmt("best max= x\n").kind,
        StmtKind::Assign {
            op: AssignOp::Max,
            ..
        }
    ));
    // A name that starts the next line is not an operator of this one.
    let src = "if c:\n    f()\ncur = fix\n";
    assert!(codes(src).is_empty());
    let stmts = stmts(src);
    assert_eq!(stmts.len(), 2);
    let StmtKind::Assign { target, value, .. } = &stmts[1].kind else {
        panic!("not an assignment")
    };
    assert_eq!(target.kind, ExprKind::Name("cur".into()));
    assert_eq!(value.kind, ExprKind::Name("fix".into()));
}

#[test]
fn an_if_let_chains_more_lets_and_conditions() {
    let src = "if let a = i - 1 and let b = j - 1 and a < b:\n    f(a, b)\n";
    assert!(codes(src).is_empty());
    let StmtKind::Expr(e) = one_stmt(src).kind else {
        panic!("not an expression")
    };
    let ExprKind::If { branches, .. } = e.kind else {
        panic!("not an if")
    };
    let Cond::Let {
        and: Some(second), ..
    } = &branches[0].0
    else {
        panic!("no first let")
    };
    let Cond::Let {
        and: Some(last), ..
    } = &**second
    else {
        panic!("no second let")
    };
    assert!(matches!(&**last, Cond::Expr(_)));
}

#[test]
fn a_match_arm_of_statements_takes_a_colon_and_a_block() {
    let src = "match cmd:\n    Install(pkg):\n        var fresh = 0\n        print(fresh)\n    Uninstall(pkg) => print(pkg)\n";
    assert!(codes(src).is_empty());
    let StmtKind::Expr(e) = one_stmt(src).kind else {
        panic!("not an expression")
    };
    let ExprKind::Match { arms, .. } = e.kind else {
        panic!("not a match")
    };
    assert_eq!(arms.len(), 2);
    assert_eq!(arms[0].body.stmts.len(), 2);
    assert_eq!(arms[1].body.stmts.len(), 1);
}

#[test]
fn a_field_or_an_operator_alone_is_a_fn_argument() {
    let src = "print(sorted(links, by: .cost), scan(a, +))\n";
    assert!(codes(src).is_empty());
    let StmtKind::Expr(e) = one_stmt(src).kind else {
        panic!("not an expression")
    };
    let ExprKind::Call { args, .. } = e.kind else {
        panic!("not a call")
    };
    for (arg, params) in [(0, 1), (1, 2)] {
        let ExprKind::Call { args, .. } = &args[arg].value.kind else {
            panic!("not a call")
        };
        let ExprKind::Lambda { params: found, .. } = &args[1].value.kind else {
            panic!("not a fn: {:?}", args[1].value.kind)
        };
        assert_eq!(found.len(), params);
    }
    // Between two values an operator is still an operator.
    assert!(codes("print(a + b, -c)\n").is_empty());
}

#[test]
fn map_literals_counts_and_chained_let_else() {
    let StmtKind::Let {
        init: Some(init), ..
    } = one_stmt("let m = [3: 4, 5: 6]\n").kind
    else {
        panic!("not a let")
    };
    assert!(matches!(init.kind, ExprKind::Map(entries) if entries.len() == 2));
    // `[n - 1]T` counts with any `Int` expression; a range keeps its `..`.
    for src in [
        "let es: [n - 1](a: Int, b: Int) = read()\n",
        "let a: [0..9]Int = 0\n",
    ] {
        assert!(codes(src).is_empty(), "{src}");
    }
    // One `let ... else` after another, sharing the `else`.
    let src = "let g = x.right and let h = y.right else: return -1\n";
    assert!(codes(src).is_empty());
    let stmts = stmts(src);
    assert_eq!(stmts.len(), 2);
    for stmt in &stmts {
        assert!(matches!(
            &stmt.kind,
            StmtKind::Let {
                otherwise: Some(_),
                ..
            }
        ));
    }
}

#[test]
fn there_are_thirty_four_keywords_and_no_others() {
    for word in crate::KEYWORDS {
        let lexed = crate::lex(&format!("{word}\n"));
        assert_ne!(
            lexed.tokens[0].kind,
            TokenKind::Ident,
            "`{word}` is reserved"
        );
    }
    for word in [
        "defer", "assume", "where", "inline", "lend", "law", "with", "by",
    ] {
        let lexed = crate::lex(&format!("{word}\n"));
        assert_eq!(lexed.tokens[0].kind, TokenKind::Ident, "`{word}` is a name");
    }
    stmts(
        "let defer = 1\nlet inline = defer + 1\nlet assume = inline\nlet by = assume\nprint(by)\n",
    );
}

#[test]
fn where_clause_bounds_parse() {
    let file = parse_file("fn f<T>(x: T) where T: Ord: print(x)\n");
    assert!(file.diagnostics.is_empty(), "{:#?}", file.diagnostics);
}

#[test]
fn removed_spellings_say_what_replaces_them() {
    for (src, needle) in [
        ("defer cur.pop()\n", "by hand"),
        ("assume x > 0\n", "assert"),
        ("inline fn f(): print(1)\n", "always inlined"),
        ("for mut x in xs: x += 1\n", "index"),
        ("if a: print(1)\nelse if b: print(2)\n", "elif"),
        ("@memo(hash)\nfn f(x: Int) -> Int = x\n", "bare `@memo`"),
        (
            "@allow(float_equality)\nfn f(x: F64) -> Bool = x == 0.5\n",
            "with lint(float_equality: allow)",
        ),
        (
            "@multitest(count: 3)\nfn solve(): print(1)\n",
            "for _ in 0..tests",
        ),
    ] {
        let diags = parse_file(src).diagnostics;
        assert_eq!(diags.len(), 1, "{src:?}: {diags:#?}");
        assert_eq!(diags[0].code.0, "E0503", "{src:?}");
        let text = format!(
            "{} {}",
            diags[0].message,
            diags[0].help.clone().unwrap_or_default()
        );
        assert!(text.contains(needle), "{src:?}: {text}");
    }
    // The v2.1 `law` and `lend` statements are still read: `spec/std/` has them.
    let diags = parse_file("struct B:\n    c: List<Int>\n    fn at(mut self, i: Int) lends mut Int:\n        law disjoint(i, j) = i != j\n        lend mut c[i]\n").diagnostics;
    assert!(diags.is_empty(), "{diags:#?}");
}

#[test]
fn a_default_can_be_a_way_out() {
    for word in ["return -1", "return", "break", "continue"] {
        let src = format!("while true:\n    let u = o ?? {word}\n");
        stmts(&src);
    }
    assert_eq!(codes("print(a + b ?? 0)\n"), Vec::<&str>::new());
    assert_eq!(codes("print(a ?? 0 + b)\n"), ["E0110"]);
    let help = parse_file("print(a ?? 0 + b)\n").diagnostics[0]
        .help
        .clone();
    assert_eq!(help.as_deref(), Some("write `(a ?? 0) + b` to use whichever is taken, or `a ?? (0 + b)` for a default that is worked out"));
    stmts("print((a ?? 0) + b, (a + b) ?? 0)\nx ??= 3\n");
}

#[test]
fn v22_statement_forms_from_the_blind_drafts_parse() {
    // A filtered loop, `;` between statements, a bodyless `@grader fn`,
    // a place-yielding `??=` and a digits spec in a string.
    assert_eq!(
        codes(
            "for x in xs if x > 0:\n    print(x)\n\
             a += 1; b += 1\n\
             for x in xs: a += x; b += 1\n\
             @grader fn get_distance(i: Int, j: Int) -> Int\n\
             (m[f(k)] ??= []).push(1)\n\
             print(\"{x:.2f}\")\n"
        ),
        Vec::<&str>::new()
    );
    // Outside `@grader` a fn still needs its body, and a `;` that ends a
    // line is still a mistake.
    assert_eq!(codes("fn f(i: Int) -> Int\n"), vec!["E0103"]);
    assert_eq!(codes("let y = 2;\n"), vec!["E0107"]);
    // A `@grader fn` whose indented body lost its `:` is one error on the
    // signature, with the body still read.
    let src = "@grader fn f(n: Int) -> Int\n    domain V = [n]\n    V.len\n";
    let parsed = parse_file(src);
    assert_eq!(codes(src), vec!["E0103"]);
    assert_eq!(parsed.diagnostics[0].fixes[0].edits[0].1, ":");
}

/// `(lo..hi) by step` is the range `lo..hi by step`; after `.rev` the
/// step is kept beside the walk, which counts down from the last value.
#[test]
fn a_range_in_parentheses_takes_a_step() {
    let step_of = |src: &str| {
        assert_eq!(codes(src), Vec::<&str>::new(), "{src}");
        let StmtKind::For { iter, .. } = stmts(src).remove(0).kind else {
            panic!("{src}")
        };
        iter.kind
    };
    for src in [
        "for i in (0..n) by 2: f(i)\n",
        "for i in ((1..=n)) by k + 1: f(i)\n",
    ] {
        assert!(
            matches!(step_of(src), ExprKind::Range { step: Some(_), .. }),
            "{src}"
        );
    }
    let ExprKind::By { walk, .. } = step_of("for i in (0..n).rev by 2: f(i)\n") else {
        panic!()
    };
    assert!(matches!(&walk.kind, ExprKind::Field { name, .. } if name.text == "rev"));
    // Anything else before `by` is not a range to step.
    assert!(!codes("for i in xs by 2: f(i)\n").is_empty());
}

#[test]
fn v03_traits_opaque_structs_mut_ref_params_and_multi_trait_impl_parse() {
    let src = "\
trait Ord(Eq): fn cmp(self, o: Self) -> Ordering
trait Hash: fn hash(self, h: &mut Hasher)
struct List<T>
struct Str
impl Read + Print for Str:
    fn read() -> Str = \"\"
";
    assert_eq!(codes(src), Vec::<&str>::new());
    let items = stmts(src);
    assert_eq!(items.len(), 6);
    assert!(matches!(&items[0].kind, StmtKind::Trait { supers, .. } if supers.len() == 1));
    assert!(matches!(
        &items[1].kind,
        StmtKind::Trait { body, .. }
            if matches!(
                &body.stmts[0].kind,
                StmtKind::Fn(f) if f.params.len() == 2 && f.params[1].mutable
            )
    ));
    assert!(
        matches!(&items[2].kind, StmtKind::Struct(s) if s.name.text == "List" && s.fields.is_empty())
    );
    assert!(
        matches!(&items[3].kind, StmtKind::Struct(s) if s.name.text == "Str" && s.fields.is_empty())
    );
    assert!(matches!(
        &items[4].kind,
        StmtKind::Impl {
            trait_name: Some(_),
            ..
        }
    ));
    assert!(matches!(
        &items[5].kind,
        StmtKind::Impl {
            trait_name: Some(_),
            ..
        }
    ));
}

#[test]
fn where_filters_are_rejected_and_suggest_if() {
    let comp = one_diag("let ys = [x for x in xs where x > 0]\n");
    assert_eq!(comp.code, cpl_diag::codes::REMOVED_SYNTAX);
    assert!(comp.message.contains("`if`"), "{comp:?}");
    assert_eq!(comp.fixes.len(), 1);
    assert_eq!(comp.fixes[0].edits[0].1, "if");

    let loop_d = one_diag("for x in xs where x > 0:\n    print(x)\n");
    assert_eq!(loop_d.code, cpl_diag::codes::REMOVED_SYNTAX);
    assert!(loop_d.message.contains("`if`"), "{loop_d:?}");
    assert_eq!(loop_d.fixes.len(), 1);
    assert_eq!(loop_d.fixes[0].edits[0].1, "if");

    let if_let = one_diag("if let x = a where x > 0:\n    print(x)\n");
    assert_eq!(if_let.code, cpl_diag::codes::REMOVED_SYNTAX);
    assert!(if_let.message.contains("`and`"), "{if_let:?}");
    assert_eq!(if_let.fixes.len(), 1);
    assert_eq!(if_let.fixes[0].edits[0].1, "and");
}

#[test]
fn multitest_wrap_lines_are_rejected_and_suggest_at_bang_multitest() {
    for src in [
        "--- multitest ---\nlet n: Int = read()\n",
        "--- with multitest() ---\nlet n: Int = read()\n",
    ] {
        let d = one_diag(src);
        assert_eq!(d.code, cpl_diag::codes::REMOVED_SYNTAX);
        assert!(d.message.contains("@!multitest"), "{d:?}");
        assert_eq!(d.fixes.len(), 1);
        assert_eq!(d.fixes[0].edits[0].1, "@!multitest");
    }
}

#[test]
fn plus_plus_is_a_lexer_error_with_plus_fix_it() {
    let parsed = parse_file("let zs = xs ++ [1] ++ ys\n");
    assert_eq!(parsed.diagnostics.len(), 2, "{:?}", parsed.diagnostics);
    for d in &parsed.diagnostics {
        assert!(d.message.contains("CP Lang concatenates with `+`"), "{d:?}");
        assert_eq!(d.fixes.len(), 1);
        assert_eq!(d.fixes[0].edits[0].1, "+");
    }
}

#[test]
fn trailing_binary_operator_continues_indented_next_line() {
    let src = "\
if a == 1 and
    b == 2 and
    c == 3:
    f()
elif d == 4 or
        e == 5:
    g()
while i < n and
    a[i] > 0:
    i += 1
if let x = p and
    let y = q and
    x < y:
    h(x, y)
let total = a +
    // middle comment
    b *
    c
";
    assert_eq!(codes(src), Vec::<&str>::new());
    assert_eq!(stmts(src).len(), 4);

    // Same-indent continuation after `and`/`or` also works when not a statement keyword.
    let same_indent = "if a == 1 and\nb == 2:\n    f()\n";
    assert_eq!(codes(same_indent), Vec::<&str>::new());
    assert_eq!(stmts(same_indent).len(), 1);

    // `>` closes generic types, so it does not continue a line.
    let gen_src = "var xs: List<Int> = []\n    let bad = 1\n";
    assert_eq!(codes(gen_src), vec!["E0006"]);

    // Breaking *before* a binary operator in a condition gives a clear hint.
    let lead = "if a == 1\n    and b == 2:\n    f()\n";
    let parsed = parse_file(lead);
    assert!(
        parsed.diagnostics.iter().any(|d| d
            .help
            .as_deref()
            .is_some_and(|h| h.contains("wrap it in `(...)`"))),
        "{:?}",
        parsed.diagnostics
    );
}

#[test]
fn one_line_lambda_body_may_assign() {
    let s = stmts("let f = (a) => cnt[a] -= 1\n");
    let StmtKind::Let { init: Some(v), .. } = &s[0].kind else {
        panic!("not a let: {:?}", s[0].kind)
    };
    let ExprKind::Lambda { body, .. } = &v.kind else {
        panic!("not a lambda: {:?}", v.kind)
    };
    assert!(matches!(&body.kind, ExprKind::Block(b) if b.stmts.len() == 1));
}

#[test]
fn generic_parameter_may_say_domain() {
    let f = fn_of("fn stops<domain I, J>(xs: [I]Int) -> Int = 0\n");
    let names: Vec<&str> = f.generics.iter().map(|g| g.text.as_str()).collect();
    assert_eq!(names, ["I", "J"]);
}

#[test]
fn generic_bound_may_end_in_a_double_angle() {
    let f = fn_of("fn dij<V, G: Next<V>>(g: G) -> Int = 0\n");
    let names: Vec<&str> = f.generics.iter().map(|g| g.text.as_str()).collect();
    assert_eq!(names, ["V", "G"]);
    assert!(codes("fn dij<V, G: Next<V>>(g: G) -> Int = 0\n").is_empty());
}

#[test]
fn record_field_may_be_an_operator_or_punned() {
    let s = stmts("let a = (count: +, taken: or)\nlet b = (t, need: n)\n");
    for st in &s {
        let StmtKind::Let { init: Some(v), .. } = &st.kind else {
            panic!("not a let: {:?}", st.kind)
        };
        assert!(
            matches!(&v.kind, ExprKind::Record(f) if f.len() == 2),
            "{:?}",
            v.kind
        );
    }
}

#[test]
fn comprehension_clauses_may_chain_with_commas() {
    let s = stmts("let x = [i + j for i in a, (j, k) in b if k]\nlet y = f(x for x in a, 3)\n");
    let StmtKind::Let { init: Some(v), .. } = &s[0].kind else {
        panic!()
    };
    let ExprKind::Comprehension { clauses, .. } = &v.kind else {
        panic!("{:?}", v.kind)
    };
    assert_eq!(clauses.len(), 2);
}

#[test]
fn pascal_case_some_and_none_read_as_the_optional_variants() {
    assert!(matches!(expr("None").kind, ExprKind::None));
    let src = "match x:\n    Some(v) => v\n    None => 0\nlet s = Side.None\n";
    assert!(codes(&format!("enum Side: Left | None\n{src}")).is_empty());
    assert_eq!(show(&expr("Some(3)")), show(&expr("some(3)")));
}

#[test]
fn tagged_variants_and_mut_lambda_parameters_parse() {
    let src = "enum Cmd:\n    @tag(\"1\") Add(x: Int)\n    @tag(\"3\") Stop\nlet f = (mut d, e) => d.push(e)\n";
    assert!(codes(src).is_empty(), "{:?}", codes(src));
}

#[test]
fn c_style_logic_and_ranges_are_reported_with_the_cp_spelling_as_a_fix() {
    let cases = [
        (
            "let a = true\nlet b = a && a\n",
            "`&&` is `and` in cp",
            "and",
        ),
        ("let a = true\nlet b = a || a\n", "`||` is `or` in cp", "or"),
        ("for i in 0..<3:\n    print(i)\n", "`..<` is `..` in cp", ""),
        ("let a: [0..<3]Int = read()\n", "`..<` is `..` in cp", ""),
        (
            "if let x = f() && x > 0:\n    print(x)\n",
            "`&&` is `and` in cp",
            "and",
        ),
        (
            "while x > 0 || y > 0:\n    x -= 1\n",
            "`||` is `or` in cp",
            "or",
        ),
    ];
    for (src, message, replacement) in cases {
        let d = one_diag(src);
        assert_eq!(d.message, message, "{src:?}");
        assert_eq!(d.fixes[0].edits[0].1, replacement, "{src:?}");
    }
    let d = one_diag("let a = true\nlet b = !a\n");
    assert_eq!(d.fixes[0].edits[0].1, "not ");
}

#[test]
fn rust_let_mut_is_reported_as_var_and_read_as_one() {
    let d = one_diag("let mut x = 1\nx += 1\n");
    assert_eq!(d.message, "`let mut` is `var` in cp");
    assert_eq!(d.fixes[0].edits[0].1, "var");
    let d = one_diag("var xs = [1]\nwhile let mut c = xs.pop():\n    print(c)\n");
    assert_eq!(d.message, "a binding in a `let` condition can't be `mut`");
    assert_eq!(d.fixes[0].edits[0].1, "");
}

#[test]
fn single_bitwise_operators_are_not_c_style_logic() {
    assert_eq!(codes("let b = 6 & 3 | 1\nlet c = 6 & &3\n").len(), 1);
    assert!(codes("let b = 6 & 3 | 1\n").is_empty());
}

#[test]
fn an_indented_if_after_eq_is_the_value_and_any_other_indented_value_is_planned() {
    assert!(codes("let a =\n    if true:\n        1\n    else:\n        2\nprint(a)\n").is_empty());
    let d = one_diag("let a =\n    1\nprint(a)\n");
    assert!(
        d.message
            .contains("a value on the indented lines after `=`"),
        "{}",
        d.message
    );
}

#[test]
fn unclosed_paren_closes_before_arrow_and_comprehension_for() {
    let src = "fn f(x: Int -> Int = x + 1\n";
    let d = one_diag(src);
    assert_eq!(d.code, cpl_diag::codes::UNCLOSED_DELIMITER);
    let (span, text) = &d.fixes[0].edits[0];
    let mut fixed = src.to_string();
    fixed.replace_range(span.start as usize..span.end as usize, text);
    assert_eq!(fixed, "fn f(x: Int) -> Int = x + 1\n");

    let src = "let ys = [x.bit(b for x in xs for b in 0..3]\n";
    let d = one_diag(src);
    assert_eq!(d.code, cpl_diag::codes::UNCLOSED_DELIMITER);
    let (span, text) = &d.fixes[0].edits[0];
    let mut fixed = src.to_string();
    fixed.replace_range(span.start as usize..span.end as usize, text);
    assert_eq!(fixed, "let ys = [x.bit(b) for x in xs for b in 0..3]\n");
}

#[test]
fn a_bad_binary_digit_is_named() {
    let d = one_diag("let x = 0b102\n");
    assert_eq!(d.code.0, "E0005");
    assert_eq!(
        d.message,
        "`0b102` isn't a number: `2` isn't a binary digit"
    );
    assert_eq!(codes("let x = 0b101\nprint(x)\n"), Vec::<&str>::new());
}

#[test]
fn an_unclosed_paren_offers_each_closer_spot() {
    let src = "let s = \"abc\"\nlet n = SuffixArray(s.distinct() + 1\nprint(n)\n";
    let d = one_diag(src);
    assert_eq!(d.code.0, "E0101");
    let titles: Vec<_> = d.fixes.iter().map(|f| f.title.as_str()).collect();
    assert_eq!(
        titles,
        [
            "insert `)`",
            "insert `)` right after `(`",
            "insert `)` after `s`",
            "insert `)` after `s.distinct()`"
        ]
    );
    let after_s = src.find("(s").unwrap() as u32 + 2;
    assert_eq!(d.fixes[2].edits[0].0, cpl_diag::Span::point(after_s));
    // Two unclosed openers need two closers: no single-spot candidates.
    let d = one_diag("let n = f(g(1\nprint(n)\n");
    assert_eq!(d.fixes.len(), 1, "{:?}", d.fixes);
}

#[test]
fn a_size_run_into_its_type_is_one_missing_bracket() {
    let src = "let n = 3\nlet a: [n - 1Int = read()\nfor x in read<[n - 1Int>():\n    print(x)\n";
    let diags = parse_file(src).diagnostics;
    let found: Vec<_> = diags
        .iter()
        .map(|d| (d.code.0, d.help.clone().unwrap()))
        .collect();
    let want = ("E0101", "add `]` before `Int`".to_string());
    assert_eq!(found, [want.clone(), want], "{diags:?}");
    assert_eq!(
        diags[0].fixes[0].edits[0].0.start as usize,
        src.find("Int ").unwrap()
    );
    assert_eq!(codes("print(0xFF)\nlet a = [1Q]\n"), ["E0005"]);
}

#[test]
fn an_unexpected_indent_moves_its_whole_run_back() {
    let src = "let a = 1\n    let b = 2\n    if b > 1:\n        print(b)\nprint(a)\n";
    let diags = parse_file(src).diagnostics;
    let mut out = src.to_string();
    for (span, text) in diags[0].fixes[0].edits.iter().rev() {
        out.replace_range(span.start as usize..span.end as usize, text);
    }
    assert_eq!(
        out,
        "let a = 1\nlet b = 2\nif b > 1:\n    print(b)\nprint(a)\n"
    );
    assert!(parse_file(&out).diagnostics.is_empty());
    let src = "fn f():\n    let a = 1\n        let b = 2\n    print(a)\nf()\n";
    let diags = parse_file(src).diagnostics;
    assert_eq!(diags[0].fixes[0].edits.len(), 1, "{diags:?}");
}

#[test]
fn a_c_increment_is_one_error_with_a_plus_equals_fix() {
    let d = parse_file("var i = 0\ni++\nxs[0]++ // bump\n").diagnostics;
    let found: Vec<_> = d
        .iter()
        .map(|d| (d.code.0, d.fixes[0].edits[0].1.as_str()))
        .collect();
    assert_eq!(found, [("E0004", " += 1"), ("E0004", " += 1")], "{d:?}");
    assert_eq!(codes("let zs = xs ++ ys\n"), ["E0004"]);
}

#[test]
fn another_languages_def_is_one_error_read_as_fn() {
    let d = parse_file(
        "def f(x: Int) -> Int:\n    return x\nfunc g():\n    print(f(1))\nlet def = 1\n",
    )
    .diagnostics;
    let found: Vec<_> = d
        .iter()
        .map(|d| (d.code.0, d.fixes[0].edits[0].1.as_str()))
        .collect();
    assert_eq!(found, [("E0503", "fn"), ("E0503", "fn")], "{d:?}");
    let d = parse_file("import std.segtree\nlet import = 1\n").diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(d[0].fixes[0].edits[0].1, "use");
}

#[test]
fn a_js_strict_equality_is_one_error_with_its_fix() {
    let d = parse_file("let a = 1\nprint(a === 1, a !== 2)\n").diagnostics;
    let found: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(found, ["==", "!="], "{d:?}");
}

#[test]
fn a_c_ternary_is_one_error_naming_the_inline_if() {
    let d = parse_file("let b = a == 1 ? 3 : 4\nprint(c ? x : y, b)\n").diagnostics;
    let found: Vec<_> = d
        .iter()
        .map(|d| (d.code.0, d.help.as_deref().unwrap_or("")))
        .collect();
    let want = ("E0503", "write `if c: a else: b`");
    assert_eq!(found, [want, want], "{d:?}");
    let fixed: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(fixed, ["if a == 1: 3 else: 4", "if c: x else: y"]);
}

#[test]
fn another_languages_type_name_is_one_error_with_ours() {
    let d = parse_file("let a: int = 5\nlet xs: vector<Int> = [a]\nlet r: 0..=n = 1\n").diagnostics;
    let found: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(found, ["Int", "List"], "{d:?}");
    let d = parse_file("let d: double = 1.5\n").diagnostics;
    assert_eq!(d[0].fixes[0].edits[0].1, "F64", "{d:?}");
}
