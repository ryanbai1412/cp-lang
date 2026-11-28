use crate::ast::{self, AstNode};
use crate::file::NodeKind;
use crate::lexer::lex;
use crate::ops::{levels, Level};
use crate::tree::{SyntaxElement, SyntaxNode};
use crate::{parse_expr, parse_stmts};

/// The tree as `Kind(child child)`, tokens by text, trivia left out.
fn shape(node: &SyntaxNode) -> String {
    let parts: Vec<String> = node
        .children_with_tokens()
        .filter_map(|e| match e {
            SyntaxElement::Node(n) => Some(shape(&n)),
            SyntaxElement::Token(t) if !t.kind().is_trivia() && !t.text().is_empty() => {
                Some(t.text().to_string())
            }
            SyntaxElement::Token(_) => None,
        })
        .collect();
    format!("{:?}({})", node.kind(), parts.join(" "))
}

fn expr(src: &str) -> (String, usize) {
    let file = parse_expr(src);
    assert_eq!(file.root.text(), src, "lossless");
    let e = file.root.children().next().unwrap();
    (shape(&e), file.diagnostics.len())
}

#[test]
fn precedence_follows_the_table() {
    let cases = [
        ("-x ** 2", "PrefixExpr(- BinExpr(NameRef(x) ** Literal(2)))"),
        ("not a == b", "PrefixExpr(not CompareExpr(NameRef(a) == NameRef(b)))"),
        ("s & (s << 1) == 0", "CompareExpr(BinExpr(NameRef(s) & ParenExpr(( BinExpr(NameRef(s) << Literal(1)) ))) == Literal(0))"),
        ("a ?? b ?? c", "BinExpr(NameRef(a) ?? BinExpr(NameRef(b) ?? NameRef(c)))"),
        ("a - b - c", "BinExpr(BinExpr(NameRef(a) - NameRef(b)) - NameRef(c))"),
        ("2 ** 3 ** 2", "BinExpr(Literal(2) ** BinExpr(Literal(3) ** Literal(2)))"),
        ("2 ** -k", "BinExpr(Literal(2) ** PrefixExpr(- NameRef(k)))"),
        ("0 <= i < n", "CompareExpr(Literal(0) <= NameRef(i) < NameRef(n))"),
        ("x not in s", "CompareExpr(NameRef(x) not in NameRef(s))"),
        ("a or b and c", "BinExpr(NameRef(a) or BinExpr(NameRef(b) and NameRef(c)))"),
        ("0..n by 2", "RangeExpr(Literal(0) .. NameRef(n) by Literal(2))"),
        ("..=n", "RangeExpr(..= NameRef(n))"),
        ("a + b is Some(v)", "BinExpr(NameRef(a) + IsExpr(NameRef(b) is PathPat(Some PatArgs(( PathPat(v) )))))"),
        ("f(x, by: 2)[i].y!?", "TryExpr(UnwrapExpr(FieldExpr(IndexExpr(CallExpr(NameRef(f) ArgList(( Arg(NameRef(x)) , Arg(by : Literal(2)) ))) IndexList([ NameRef(i) ])) . y) !) ?)"),
        ("best max= x + 1", "KeepExpr(NameRef(best) max = BinExpr(NameRef(x) + Literal(1)))"),
        ("(a, b) => a + b", "LambdaExpr(ParamList(( PathPat(a) , PathPat(b) )) => BinExpr(NameRef(a) + NameRef(b)))"),
        ("(a,)", "TupleExpr(( NameRef(a) , ))"),
    ];
    for (src, want) in cases {
        assert_eq!(expr(src), (want.to_string(), 0), "{src}");
    }
}

#[test]
fn a_broken_operand_keeps_the_rest_of_the_expression() {
    // The missing operand is a zero-width node; the call, its other
    // arguments and the operator after it stay in the tree.
    let (tree, errors) = expr("f(a +, b) * 2");
    assert_eq!(errors, 1, "{tree}");
    assert_eq!(
        tree,
        "BinExpr(CallExpr(NameRef(f) ArgList(( Arg(BinExpr(NameRef(a) + Missing())) , Arg(NameRef(b)) ))) * Literal(2))"
    );
    // A token that cannot start an operand is skipped into an `Error` node.
    let (tree, errors) = expr("[1, ., 3]");
    assert_eq!(
        (tree.as_str(), errors),
        ("ListExpr([ Literal(1) , Error(.) , Literal(3) ])", 1)
    );
    // A broken pattern ends at the closer; the call around it survives.
    let (tree, errors) = expr("g(x is , y)");
    assert_eq!(errors, 1, "{tree}");
    assert!(tree.contains("Arg(NameRef(y))"), "{tree}");
    // The lexer closes an unclosed bracket with a zero-width `)` and one
    // error; the list reads as closed.
    let (tree, errors) = expr("h(1, 2");
    assert_eq!(
        (tree.as_str(), errors),
        (
            "CallExpr(NameRef(h) ArgList(( Arg(Literal(1)) , Arg(Literal(2))))",
            1
        )
    );
}

#[test]
fn trivia_sit_between_nodes() {
    let file = parse_expr("a  // c\n  + b");
    let bin = file.root.children().next().unwrap();
    assert_eq!(bin.kind(), NodeKind::BinExpr);
    for child in bin.children() {
        let text = child.text();
        assert_eq!(
            text.trim(),
            text,
            "a node starts and ends at a token: {text:?}"
        );
    }
}

/// The spec's §1.5 list is the operator table: each line's operators sit
/// at one level, and the levels fall line by line.
#[test]
fn the_table_is_the_specs_precedence_list() {
    let spec = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../spec/core/01-lexical-grammar.md"
    ))
    .unwrap();
    let section = &spec[spec.find("## 1.5").unwrap()..spec.find("## 1.6").unwrap()];
    let mut last: Option<Level> = None;
    let mut lines = 0;
    for line in section.lines() {
        let Some((n, rest)) = line.split_once(". ") else {
            continue;
        };
        if n.parse::<u32>().is_err() {
            continue;
        }
        // The operators are the backticked words before any aside.
        let ops: Vec<&str> = rest
            .split(" (")
            .next()
            .unwrap()
            .split('`')
            .skip(1)
            .step_by(2)
            .collect();
        let sets: Vec<Vec<Level>> = ops
            .iter()
            .filter_map(|op| {
                let lexed = lex(op);
                let kinds: Vec<_> = lexed
                    .tokens
                    .iter()
                    .map(|t| t.kind)
                    .filter(|k| {
                        !matches!(
                            k,
                            crate::TokenKind::Newline
                                | crate::TokenKind::Indent
                                | crate::TokenKind::Dedent
                                | crate::TokenKind::Eof
                        )
                    })
                    .collect();
                (kinds.len() == 1)
                    .then(|| levels(kinds[0]))
                    .filter(|l| !l.is_empty())
            })
            .collect();
        let Some(first) = sets.first() else { continue };
        // Each line's operators share a level below the line above's: `-`
        // is `Add` and `Unary`, and its line picks which.
        let below = |l: &Level| last.is_none_or(|prev| *l < prev);
        let level = first
            .iter()
            .copied()
            .filter(|l| below(l) && sets.iter().all(|s| s.contains(l)))
            .max();
        lines += 1;
        let Some(level) = level else {
            panic!("line {n}: no level below the line above shared by {sets:?}")
        };
        let levels = [level];
        last = Some(levels[0]);
    }
    assert!(lines >= 13, "read {lines} operator lines");
}

fn stmts(src: &str) -> (Vec<String>, usize) {
    let file = parse_stmts(src);
    assert_eq!(file.root.text(), src, "lossless");
    (
        file.root.children().map(|n| shape(&n)).collect(),
        file.diagnostics.len(),
    )
}

#[test]
fn statements_and_blocks() {
    let (got, errors) = stmts(
        "let (a, b): (Int, [N]?Vec<Vec<Int>>) = f(x)\nbest max= a\nfor i in 0..n if i % 2 == 0, j in xs:\n    s += i\nelse: s = 0\nif let Some(v) = m, v > 0: return v\nelif q: break\nelse:\n    match v:\n        0 | 1 => x\n        _ if y => z = 1\n",
    );
    assert_eq!(errors, 0, "{got:#?}");
    let kinds: Vec<&str> = got.iter().map(|s| s.split('(').next().unwrap()).collect();
    assert_eq!(kinds, ["LetStmt", "ExprStmt", "ForStmt", "IfStmt"]);
    assert!(
        got[0].contains("GenericArgs(< PathType(Vec GenericArgs(< PathType(Int) >>))))"),
        "{}",
        got[0]
    );
    assert!(
        got[2].contains("ElseClause(else Block(: AssignStmt(NameRef(s) = Literal(0))))"),
        "{}",
        got[2]
    );
    assert!(
        got[3].contains(
            "MatchArm(WildcardPat(_) if NameRef(y) => AssignStmt(NameRef(z) = Literal(1)))"
        ),
        "{}",
        got[3]
    );
}

#[test]
fn a_broken_statement_keeps_its_block_and_neighbours() {
    // A broken `while` condition is one error: its block and the
    // statements around it, in the same block, are all in the tree.
    let (got, errors) = stmts("x = 1\nwhile a + :\n    y = 2\n    z =\n    w = 3\nv = 4\n");
    assert_eq!(errors, 2, "{got:#?}");
    assert_eq!(got.len(), 3, "{got:#?}");
    assert!(
        got[1].contains("BinExpr(NameRef(a) + Missing())"),
        "{}",
        got[1]
    );
    assert!(
        got[1].contains("AssignStmt(NameRef(y) = Literal(2))"),
        "{}",
        got[1]
    );
    assert!(
        got[1].contains("AssignStmt(NameRef(w) = Literal(3))"),
        "{}",
        got[1]
    );
    assert!(
        got[1].contains("AssignStmt(NameRef(z) = Missing())"),
        "{}",
        got[1]
    );
    assert_eq!(got[2], "AssignStmt(NameRef(v) = Literal(4))");
}

#[test]
fn items_carry_their_decorators_and_members() {
    let src = "@!fast_io\nuse std.algebra.{Monoid, Group}\n@inline\n@cache(size: 4)\npub fn gcd<T: Eq + Ord, const N: Int>(a: T, b: T; mut k = 1) -> T where T: Copy = a\nstruct P:\n    x: Int\n    pub var y: ?Int = none\n    fn norm(self) -> Int: return self.x\nenum Dir = Left | Right\ntrait Op<T>(Eq):\n    type Out: Eq\n    law assoc(a: T, b: T) where T: Eq = a == b\n    fn op(self, other: T) -> T\nimpl Op<Int> for P where P: Eq:\n    type Out = Int\n    fn op(self, other: Int) -> Int = other\ndomain N = n + 1\nlet (domain M, i) = f()\nextern fn sqrt(x: Float) -> Float\nfn gen(n: Int) -> yields Int:\n    yield n\n";
    let (got, errors) = stmts(src);
    assert_eq!(errors, 0, "{got:#?}");
    let kinds: Vec<&str> = got
        .iter()
        .map(|s| s.split('(').next().unwrap_or(""))
        .collect();
    assert_eq!(
        kinds,
        [
            "Directive",
            "UseDecl",
            "FnDecl",
            "StructDecl",
            "EnumDecl",
            "TraitDecl",
            "ImplDecl",
            "DomainDecl",
            "DomainDecl",
            "ExternDecl",
            "FnDecl"
        ]
    );
    assert!(
        got[2].starts_with("FnDecl(Decorator(@ inline) Decorator(@ cache ArgList("),
        "{}",
        got[2]
    );
    assert!(
        got[2].contains("WhereClause(where WherePred(PathType(T) : Bound(PathType(Copy))))"),
        "{}",
        got[2]
    );
    assert!(
        got[5].contains("LawDecl(law Name(assoc) ParamList("),
        "{}",
        got[5]
    );
    assert!(
        got[5].contains(
            "WhereClause(where WherePred(PathType(T) : Bound(PathType(Eq)))) = CompareExpr("
        ),
        "{}",
        got[5]
    );
}

#[test]
fn a_broken_member_keeps_its_siblings_and_the_next_item() {
    let (got, errors) = stmts("struct P:\n    x: Int\n    y: )\n    z: Int\nfn f() = 1\n");
    assert!(errors >= 1, "{got:#?}");
    assert_eq!(got.len(), 2, "{got:#?}");
    assert!(
        got[0].contains("FieldDecl(Name(x) : PathType(Int))"),
        "{}",
        got[0]
    );
    assert!(
        got[0].contains("FieldDecl(Name(z) : PathType(Int))"),
        "{}",
        got[0]
    );
    assert!(got[1].starts_with("FnDecl(fn Name(f)"), "{}", got[1]);
}

#[test]
fn leading_trivia_is_inside_the_file() {
    for src in ["//! docs\n", "  // c\nx = 1\n", "\n\nf(x)"] {
        assert_eq!(parse_stmts(src).root.text(), src);
    }
    assert_eq!(parse_expr("  a + b").root.text(), "  a + b");
}

#[test]
fn a_colon_arm_is_one_error_and_keeps_its_body() {
    let (got, errors) = stmts("match k:\n    0: yield 1\n    _ => k\n");
    assert_eq!(errors, 1, "{got:#?}");
    let diags = parse_stmts("match k:\n    0: yield 1\n").diagnostics;
    assert_eq!(diags[0].fixes.len(), 1, "{diags:?}");
    assert!(
        got[0].contains("MatchArm(LiteralPat(0) Block(: YieldStmt(yield Literal(1))))"),
        "{}",
        got[0]
    );
}

#[test]
fn a_stray_indent_among_arms_ends() {
    // No arm can start at the indent, so the arm loop must still advance.
    let (got, errors) = stmts("match k:\n    _ x\n        y\nz = 1\n");
    assert!(errors >= 1, "{got:#?}");
    assert_eq!(
        got.last().map(String::as_str),
        Some("AssignStmt(NameRef(z) = Literal(1))")
    );
}

#[test]
fn collection_record_generic_and_operator_forms_parse() {
    let src = "let a = [x * x for x in xs if x > 0]\nlet m = [1: 10, 2: 20]\nlet e = [:]\nlet r = (g: 1, x)\nvar q = Deque<T>.new()\nlet s = fold(+, xs)\nlet t = [R, C] (r, c) => r\nlet f = [n]0\nlet k = sum(f(x) for x in xs)\nlet v = @intrinsic(\"popcount\")\nlet w = { domain V = 0..3; Int(V(1)) }\n";
    let (got, errors) = stmts(src);
    assert_eq!(errors, 0, "{got:#?}");
    for (i, want) in [
        "CompExpr([ BinExpr(",
        "MapExpr([ MapEntry(Literal(1) : Literal(10))",
        "MapExpr([ : ])",
        "RecordExpr(( RecordField(g : Literal(1)) , NameRef(x) ))",
        "GenericExpr(NameRef(Deque) GenericArgs(< PathType(T) >))",
        "Arg(OpValue(+))",
        "TableExpr(Dims([ PathType(R) , PathType(C) ]) ParamList(",
        "TableFill(ListExpr([ NameRef(n) ]) Literal(0))",
        "GeneratorExpr(CallExpr(",
        "IntrinsicExpr(@ intrinsic ( \"popcount\" ))",
        "BlockExpr({ DomainDecl(",
    ]
    .iter()
    .enumerate()
    {
        assert!(got[i].contains(want), "{}: {}", want, got[i]);
    }
}

#[test]
fn a_lowercase_name_takes_type_arguments_only_before_a_call() {
    let (got, errors) = stmts("let c = a < b > (d)\nlet n = read<Int>()\n");
    assert_eq!(errors, 0, "{got:#?}");
    assert!(!got[0].contains("GenericExpr"), "{}", got[0]);
    assert!(got[1].contains("GenericExpr(NameRef(read)"), "{}", got[1]);
}

#[test]
fn places_punning_field_fns_with_and_record_patterns_parse() {
    let src = "let p = mut m[k]\nlet h = Heap.new(by:)\nlet b = xs.min(by: .0)\nlet s = t with base: lo, n: hi - lo\nlet (mat: got, rank) = m.rref()\nlet r = if ok: 1 else: return None\nvar ar = PArena<PMultiset<0..8>>()\n";
    let (got, errors) = stmts(src);
    assert_eq!(errors, 0, "{got:#?}");
    for (i, want) in [
        "MutExpr(mut IndexExpr(",
        "Arg(by :)",
        "Arg(by : FieldFn(. 0))",
        "WithExpr(NameRef(t) with RecordField(base : NameRef(lo)) , RecordField(n : ",
        "RecordPat(( RecordPatField(mat : PathPat(got)) , PathPat(rank) ))",
        "ReturnStmt(return ",
        "GenericExpr(NameRef(PArena) GenericArgs(< PathType(PMultiset GenericArgs(< RangeType(",
    ]
    .iter()
    .enumerate()
    {
        assert!(got[i].contains(want), "{}: {}", want, got[i]);
    }
}

#[test]
fn lend_statements_and_bound_conditions_parse() {
    let src = "lend mut self.buf[i]\nlend data[k]\nif M: Selective<T> + Eq:\n    x\nif ok: x\n";
    let (got, errors) = stmts(src);
    assert_eq!(errors, 0, "{got:#?}");
    assert!(
        got[0].starts_with("LendStmt(lend mut IndexExpr("),
        "{}",
        got[0]
    );
    assert!(got[1].starts_with("LendStmt(lend IndexExpr("), "{}", got[1]);
    assert!(
        got[2].contains("Condition(BoundCond(NameRef(M) : Bound("),
        "{}",
        got[2]
    );
    assert!(!got[3].contains("BoundCond"), "{}", got[3]);
}

#[test]
fn an_unnamed_payload_is_reported_and_kept() {
    let src = "enum E = One(T) | Two(Int, b: Str) | Three(a, b: Int)\n";
    let file = crate::parse(src);
    let fixes: Vec<_> = (file.diagnostics.iter())
        .map(|d| {
            d.fixes
                .iter()
                .map(|f| f.edits[0].1.clone())
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        fixes,
        [vec!["v: ".to_string()], vec!["v: ".into()], vec![]],
        "{:?}",
        file.diagnostics
    );
    let item = file.items().next().unwrap();
    let fields = item
        .descendants()
        .filter(|n| n.kind() == NodeKind::FieldDecl)
        .count();
    assert_eq!(fields, 5);
    assert_eq!(file.root.text(), src);
}

#[test]
fn var_params_parse_and_lazy_and_amp_mut_are_reported() {
    let src = "fn f(var xs: List<Int>, mut y: Int, lazy z: Int, w: &mut Int):\n    xs.len\n";
    let file = crate::parse(src);
    assert_eq!(file.root.text(), src, "lossless");
    let params = file
        .root
        .descendants()
        .filter(|n| n.kind() == NodeKind::Param)
        .count();
    assert_eq!(params, 4);
    let at = |s: cpl_diag::Span| &src[s.start as usize..s.end as usize];
    let fixes: Vec<_> = file
        .diagnostics
        .iter()
        .map(|d| d.fixes[0].edits.clone())
        .collect();
    assert_eq!(fixes.len(), 2, "{:?}", file.diagnostics);
    assert_eq!((at(fixes[0][0].0), fixes[0][0].1.as_str()), ("lazy ", ""));
    assert_eq!((at(fixes[1][0].0), fixes[1][0].1.as_str()), ("&mut ", ""));
    let w = src.find("w:").unwrap() as u32;
    assert_eq!((fixes[1][1].0.start, fixes[1][1].1.as_str()), (w, "mut "));
}

#[test]
fn a_variant_value_is_reported_and_the_variant_kept() {
    use crate::ast::{self, AstNode};
    let src = "enum C:\n    Red = 2\n    Green\n    @tag(\"b\") Blue = -7\nenum D = A = 1 | B\n";
    let file = crate::parse(src);
    assert_eq!(file.root.text(), src, "lossless");
    let codes: Vec<_> = file.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(
        codes,
        [cpl_diag::codes::NOT_IMPLEMENTED; 3],
        "{:#?}",
        file.diagnostics
    );
    let at = |d: &cpl_diag::Diagnostic| &src[d.span.start as usize..d.span.end as usize];
    assert!(file.diagnostics.iter().all(|d| at(d) == "="));
    let names: Vec<_> = file
        .root
        .descendants()
        .filter_map(ast::Variant::cast)
        .map(|v| v.name().unwrap().text().to_string())
        .collect();
    assert_eq!(names, ["Red", "Green", "Blue", "A", "B"]);
}

#[test]
fn a_lambda_param_and_its_return_are_types() {
    let src = "let f = (a: ?W, b: [V]Int) -> ?W => a\n";
    let file = crate::parse(src);
    assert!(file.diagnostics.is_empty(), "{:#?}", file.diagnostics);
    let kinds: Vec<_> = file.root.descendants().map(|n| n.kind()).collect();
    let optionals = kinds
        .iter()
        .filter(|k| **k == crate::NodeKind::OptionalType)
        .count();
    assert_eq!(optionals, 2);
}

#[test]
fn an_inline_if_continues_on_deeper_elif_and_else_lines() {
    let src = "fn f(k: Int) -> Int:\n    let x = if k == 0: 1\n        elif k == 1: 2\n        else: 3\n    if k > 0: x\n    else: 0\n";
    let file = crate::parse(src);
    assert!(file.diagnostics.is_empty(), "{:?}", file.diagnostics);
    let count = |k| file.root.descendants().filter(|n| n.kind() == k).count();
    assert_eq!(
        (count(NodeKind::ElifClause), count(NodeKind::ElseClause)),
        (1, 2)
    );
    let closed = "fn f(k: Int) -> Int:\n    let x = if k == 0: 1 else: 2\n        else: 3\n    x\n";
    assert!(!crate::parse(closed).diagnostics.is_empty());
}

#[test]
fn and_after_a_let_condition_is_reported_with_a_comma_fix() {
    let src = "if let i = f(a and b) and i > 1: g()\nif a and b: g()\n";
    let file = crate::parse(src);
    assert_eq!(file.root.text(), src, "lossless");
    assert_eq!(file.diagnostics.len(), 1, "{:?}", file.diagnostics);
    let (span, text) = file.diagnostics[0].fixes[0].edits[0].clone();
    assert_eq!(
        (&src[span.start as usize..span.end as usize], text.as_str()),
        (" and", ",")
    );
    let conds = file
        .root
        .descendants()
        .filter(|n| n.kind() == NodeKind::LetCond)
        .count();
    assert_eq!(conds, 1);
}

#[test]
fn a_yields_param_is_reported_with_a_fix_that_deletes_it() {
    let src = "fn f<S>(self: yields Slice<S>) -> yields Int:\n    0\n";
    let file = crate::parse(src);
    assert_eq!(file.diagnostics.len(), 1, "{:?}", file.diagnostics);
    let (span, text) = file.diagnostics[0].fixes[0].edits[0].clone();
    assert_eq!(
        (&src[span.start as usize..span.end as usize], text.as_str()),
        ("yields ", "")
    );
}

#[test]
fn guard_let_is_reported_and_the_let_else_kept() {
    let src = "fn f(r: ?Int) -> Int:\n    guard let x = r else: return 0\n    x\n";
    let file = crate::parse(src);
    assert_eq!(file.root.text(), src, "lossless");
    assert_eq!(file.diagnostics.len(), 1, "{:?}", file.diagnostics);
    let (span, text) = file.diagnostics[0].fixes[0].edits[0].clone();
    assert_eq!(
        (&src[span.start as usize..span.end as usize], text.as_str()),
        ("guard ", "")
    );
}

#[test]
fn a_clause_filter_takes_let_binders_and_commas() {
    let src = "for s in 0..2 if let c = kid[s], c > 0, j in 0..c, (a, b) in ps:\n    print(j)\nlet q = [y for x in xs if let y = f(x) if y > 1]\nlet r = [(x, y) for x in xs if x > 2, x < 4, y in xs if y < 2]\nprint(sum(x for x in xs, (a, b) in ps), 3)\n";
    let file = crate::parse(src);
    assert!(file.diagnostics.is_empty(), "{:?}", file.diagnostics);
    let count = |k| file.root.descendants().filter(|n| n.kind() == k).count();
    assert_eq!(count(NodeKind::ForClause), 3);
    assert_eq!(count(NodeKind::CompClause), 5);
    assert_eq!(count(NodeKind::LetCond), 2);
}

#[test]
fn a_second_for_in_a_comprehension_is_reported_with_a_comma_fix() {
    let src = "let ps = [(i, j) for i in 0..3 for j in 0..2]\n";
    let file = crate::parse(src);
    assert_eq!(file.root.text(), src, "lossless");
    assert_eq!(file.diagnostics.len(), 1, "{:?}", file.diagnostics);
    let (span, text) = file.diagnostics[0].fixes[0].edits[0].clone();
    assert_eq!(
        (&src[span.start as usize..span.end as usize], text.as_str()),
        (" for", ",")
    );
    let count = |k| file.root.descendants().filter(|n| n.kind() == k).count();
    assert_eq!(count(NodeKind::CompClause), 2);
}

#[test]
fn trusted_if_const_record_pattern_shorthand_var_and_holes_parse() {
    let src = "@trusted:\n    y = 6
if const verbose:\n    print(1)\nelse:\n    print(2)\nfor (u:, v:) in es:\n    print(u)\nlet (w:, ..) = e\nfor var x in xs:\n    x += 1\nimpl Len for _..=_:\n    fn len(self) -> Int = 0\n";
    let file = crate::parse(src);
    assert!(file.diagnostics.is_empty(), "{:?}", file.diagnostics);
    assert_eq!(file.root.text(), src, "lossless");
    let count = |k| file.root.descendants().filter(|n| n.kind() == k).count();
    assert_eq!(count(NodeKind::TrustedStmt), 1);
    assert_eq!(count(NodeKind::RecordPatField), 3);
    assert_eq!(count(NodeKind::RecordPatRest), 1);
    assert_eq!(count(NodeKind::HoleExpr), 2);
    let ifs: Vec<_> = file
        .root
        .descendants()
        .filter_map(ast::IfStmt::cast)
        .collect();
    assert!(ifs[0].is_const());
    let pats: Vec<_> = file
        .root
        .descendants()
        .filter_map(ast::RecordPat::cast)
        .collect();
    assert_eq!((pats[0].has_rest(), pats[1].has_rest()), (false, true));
    assert!(pats[0].fields().all(|f| f.pattern().is_none()));
    assert!(file
        .root
        .descendants()
        .filter_map(ast::PathPat::cast)
        .any(|p| p.is_var()));
}

#[test]
fn do_before_a_block_is_reported_with_a_fix_that_deletes_it() {
    let src = "let f = x => do:\n    let y = x\n    y\nprint(g(m => do:\n    m\n))\n";
    let file = crate::parse(src);
    assert_eq!(file.root.text(), src, "lossless");
    assert_eq!(file.diagnostics.len(), 2, "{:?}", file.diagnostics);
    let (span, text) = file.diagnostics[0].fixes[0].edits[0].clone();
    assert_eq!(
        (&src[span.start as usize..span.end as usize], text.as_str()),
        (" do:", "")
    );
    let fixed = "let f = x =>\n    let y = x\n    y\nprint(g(m =>\n    m\n))\n";
    assert!(crate::parse(fixed).diagnostics.is_empty());
}

#[test]
fn a_lowercase_generic_before_an_end_takes_type_arguments_and_a_comparison_stays() {
    let src = "let g = id<Int>\nlet h = f(id<Str>, 1)\nlet c = a < b > c\nlet d = x < y\n";
    let file = crate::parse(src);
    assert_eq!(file.root.text(), src, "lossless");
    let count = |k| file.root.descendants().filter(|n| n.kind() == k).count();
    assert_eq!(count(NodeKind::GenericExpr), 2, "{:?}", file.diagnostics);
}

#[test]
fn turbofish_fn_keyword_types_and_bare_lends_are_reported_with_fixes() {
    let cases = [
        ("let g = id::<Int>\n", "let g = id<Int>\n"),
        ("let x = read::<Int>()\n", "let x = read<Int>()\n"),
        ("let s = xs.sum::<Int>()\n", "let s = xs.sum<Int>()\n"),
        ("fn t(f: fn(Int) -> Int) = 0\n", "fn t(f: Int -> Int) = 0\n"),
        (
            "fn t(f: fn(Int, Int) -> Int) = 0\n",
            "fn t(f: (Int, Int) -> Int) = 0\n",
        ),
        (
            "fn t(xs: [V]Int, l: V) lends [V]Int = xs[l..]\n",
            "fn t(xs: [V]Int, l: V) -> lends [V]Int = xs[l..]\n",
        ),
        (
            "fn at(mut self, i: Int) lends mut Int = self.a[i]\n",
            "fn at(mut self, i: Int) -> lends mut Int = self.a[i]\n",
        ),
    ];
    for (src, want) in cases {
        let file = crate::parse(src);
        assert_eq!(file.root.text(), src, "lossless");
        assert_eq!(file.diagnostics.len(), 1, "{src}: {:?}", file.diagnostics);
        let (fixed, applied) = cpl_diag::splice_fixes(src, &file.diagnostics);
        assert_eq!((fixed.as_str(), applied), (want, 1), "{src}");
        assert!(crate::parse(want).diagnostics.is_empty(), "{want}");
    }
}

#[test]
fn prefix_op_fns_struct_lets_value_type_args_and_layout_in_braces_parse() {
    let srcs = [
        "fn -(a: V) -> V = V(-a.x, -a.y)\n",
        "struct K:\n    x: Int\n    let BASE = 5\n    fn base(self) -> Int = BASE\n",
        "type Z = Mod<1e9 + 7>\n",
        "fn f(n: Int) -> [2 * n, 2 * n]1..=n = g()\n",
        "let xs: List<?0..=255> = [Some(255), None]\n",
        "let g = \"ab cd\".parse<[2, 2]Char>()!\n",
        "fn blk(n: Int) -> Int:\n    let v = {\n        if n > 5:\n            return 99\n        n * 2\n    }\n    v\n",
        "let w = match i % 3:\n    0 => continue\n    _ => {\n        if i == 5:\n            break\n        i * 100\n    }\n",
    ];
    for src in srcs {
        let file = crate::parse(src);
        assert_eq!(file.root.text(), src, "lossless");
        assert!(file.diagnostics.is_empty(), "{src}: {:?}", file.diagnostics);
    }
    let file = crate::parse(srcs[0]);
    let op = file
        .root
        .descendants()
        .find_map(crate::ast::OpFnName::cast)
        .unwrap();
    assert!(op.is_prefix());
    assert_eq!(op.op().map(|t| t.kind()), Some(crate::TokenKind::Minus));
    let bin = crate::parse("fn (a: V) - (b: V) -> V = a\n");
    let op = bin
        .root
        .descendants()
        .find_map(crate::ast::OpFnName::cast)
        .unwrap();
    assert!(!op.is_prefix());
}

#[test]
fn a_modules_role_sits_above_its_first_use() {
    let file = crate::parse("//! The prelude.\n@role(prelude)\nuse std.io\n\nfn f() -> Int = 1\n");
    assert!(file.diagnostics.is_empty(), "{:?}", file.diagnostics);
    let roles: Vec<_> = file
        .decorators()
        .iter()
        .map(|d| d.syntax().text())
        .collect();
    assert_eq!(roles, ["@role(prelude)"]);
    assert!(crate::parse("@role(convert)\ntrait T:\n    fn f(self)\n")
        .decorators()
        .is_empty());
}

#[test]
fn every_decorator_row_cites_a_rule_the_spec_defines() {
    use crate::grammar::{Decorator, DecoratorKind};
    let core = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/core");
    let spec: String = std::fs::read_dir(core)
        .unwrap()
        .map(|e| std::fs::read_to_string(e.unwrap().path()).unwrap_or_default())
        .collect();
    for rule in Decorator::rules() {
        assert!(
            spec.contains(&format!("({rule})")) || spec.contains(&format!("({rule},")),
            "{rule}"
        );
    }
    assert_eq!(Decorator::of("memo"), Some(Decorator::Memo));
    assert_eq!(Decorator::live("tags"), None);
    assert_eq!(Decorator::Multitest.kind(), DecoratorKind::Directive);
    assert_eq!(Decorator::of("record"), None);
}
