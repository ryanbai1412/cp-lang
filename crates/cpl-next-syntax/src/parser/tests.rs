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
        got[2].contains("WhereClause(where WherePred(T : Bound(PathType(Copy))))"),
        "{}",
        got[2]
    );
    assert!(
        got[5].contains("LawDecl(law assoc ParamList("),
        "{}",
        got[5]
    );
    assert!(
        got[5].contains("WhereClause(where WherePred(T : Bound(PathType(Eq)))) = CompareExpr("),
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
        got[0].contains("FieldDecl(x : PathType(Int))"),
        "{}",
        got[0]
    );
    assert!(
        got[0].contains("FieldDecl(z : PathType(Int))"),
        "{}",
        got[0]
    );
    assert!(got[1].starts_with("FnDecl(fn f"), "{}", got[1]);
}

#[test]
fn leading_trivia_is_inside_the_file() {
    for src in ["//! docs\n", "  // c\nx = 1\n", "\n\nf(x)"] {
        assert_eq!(parse_stmts(src).root.text(), src);
    }
    assert_eq!(parse_expr("  a + b").root.text(), "  a + b");
}

#[test]
fn match_arms_take_a_block_after_colon() {
    let (got, errors) = stmts("match k:\n    0: yield 1\n    _:\n        yield k\n");
    assert_eq!(errors, 0, "{got:#?}");
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
