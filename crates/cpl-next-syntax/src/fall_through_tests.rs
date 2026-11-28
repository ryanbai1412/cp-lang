use crate::ast::{self, AstNode};
use crate::file::NodeKind;
use crate::visit::{leaves_from_parts, leaves_out_of, mentions_any, pattern_names};

fn first<N: AstNode>(src: &str, kind: NodeKind) -> (crate::SyntaxFile, N) {
    let file = crate::parse(src);
    assert_eq!(file.root.text(), src, "lossless");
    assert!(file.diagnostics.is_empty(), "{src}: {:?}", file.diagnostics);
    let n = file
        .root
        .descendants()
        .filter(|n| n.kind() == kind)
        .find_map(N::cast)
        .unwrap();
    (file, n)
}

/// The name these tests' `@role(todo)` fn has; the checker resolves it.
const TODO: &str = "todo";

fn todo(n: &ast::NameRef) -> bool {
    n.ident().is_some_and(|t| t.text() == TODO)
}

fn end(src: &str) -> Option<(String, &'static str)> {
    let (_, f) = first::<ast::FnDecl>(src, NodeKind::FnDecl);
    let (span, how) = f.body()?.falls_through(&todo)?;
    Some((src[span.start as usize..span.end as usize].to_string(), how))
}

#[test]
fn a_body_falls_through_where_its_last_statement_can_end() {
    let no_else = Some(("if".to_string(), "this `if` has no `else`"));
    let can_end = |kw: &str| Some((kw.to_string(), "this loop can end"));
    let cases = [
        ("fn f() -> Int:\n    if c:\n        return 1\n", no_else.clone()),
        ("fn f() -> Int:\n    if c:\n        return 1\n    elif d:\n        return 2\n    else:\n        return 3\n", None),
        ("fn f() -> Int:\n    if c:\n        if d:\n            return 1\n    else:\n        return 3\n", Some(("if".to_string(), "this `if` has no `else`"))),
        ("fn f() -> Int:\n    while true:\n        x += 1\n", None),
        ("fn f() -> Int:\n    while true:\n        if c:\n            break\n", can_end("while")),
        ("fn f() -> Int:\n    while true:\n        for i in xs:\n            break\n", None),
        ("fn f() -> Int:\n    while c:\n        x += 1\n", can_end("while")),
        ("fn f() -> Int:\n    for i in xs:\n        return i\n", can_end("for")),
        ("fn f() -> Int:\n    for i in xs:\n        return i\n    else:\n        return 0\n", None),
        ("fn f() -> Int:\n    unchecked:\n        while c:\n            x += 1\n", can_end("while")),
        ("fn f() -> Int:\n    let x = 1\n    return x\n", None),
        ("fn f() -> Int = 1\n", None),
    ];
    for (src, want) in cases {
        assert_eq!(end(src), want, "{src}");
    }
}

#[test]
fn a_block_never_ends_when_it_leaves_or_loops_forever() {
    let never = |src: &str| {
        let (_, f) = first::<ast::FnDecl>(src, NodeKind::FnDecl);
        f.body().unwrap().never_ends(&todo)
    };
    assert!(never("fn f() -> Int:\n    return 1\n"));
    assert!(never("fn f() -> Int:\n    todo\n"));
    assert!(never("fn f() -> Int:\n    assert false\n"));
    assert!(never("fn f() -> Int:\n    while true:\n        x += 1\n"));
    assert!(never(
        "fn f() -> Int:\n    if c:\n        return 1\n    else:\n        todo\n"
    ));
    assert!(!never("fn f() -> Int:\n    if c:\n        return 1\n"));
    assert!(!never("fn f() -> Int:\n    while true:\n        break\n"));
    assert!(!never("fn f() -> Int:\n    x\n"));
}

#[test]
fn a_bare_try_is_one_no_coalesce_catches() {
    let bare = |src: &str| {
        let file = crate::parse_expr(src);
        let e = file.root.descendants().find_map(ast::Expr::cast).unwrap();
        e.bare_try()
            .map(|s| src[s.start as usize..s.end as usize].to_string())
    };
    assert_eq!(bare("f(a?) ?? b"), None);
    assert_eq!(bare("g(x, h(y?))").as_deref(), Some("y?"));
    assert_eq!(bare("a? + b?").as_deref(), Some("a?"));
    assert_eq!(bare("xs[i?].len"), Some("i?".to_string()));
    assert_eq!(bare("(a < b?)"), Some("b?".to_string()));
    assert_eq!(bare("f(x)"), None);
}

#[test]
fn visit_reads_names_and_leaves_off_descendants() {
    let src = "fn f(x: Int) -> Int:
    match x:
        (a, Some(b), self, Red) => return a
        A(p) | B(p) => break
        _ =>
            for i in xs:
                break
                continue
            y + 1
";
    let (_, m) = first::<ast::MatchExpr>(src, NodeKind::MatchExpr);
    let arms: Vec<_> = m.arms().collect();
    let names = |i: usize| {
        let mut out = Vec::new();
        pattern_names(&arms[i].pattern().unwrap(), &mut out);
        out
    };
    assert_eq!(names(0), ["a", "b", "self"]);
    assert_eq!(names(1), ["p"]);
    assert!(names(2).is_empty());
    let leaves: Vec<_> = leaves_out_of(&ast::Expr::Match(m.clone()))
        .into_iter()
        .map(|s| &src[s.start as usize..s.end as usize])
        .collect();
    assert_eq!(leaves, ["return a", "break"]);
    assert!(mentions_any(m.syntax(), &["y"]));
    assert!(mentions_any(m.syntax(), &["xs", "zz"]));
    assert!(
        !mentions_any(m.syntax(), &["i", "zz"]),
        "a pattern binds, it doesn't mention"
    );
}

/// A leave inside an operand, argument or condition is from a part of a value; one in an arm of
/// a statement's whole `if` or `match`, or of a lambda's body, is the statement's own.
#[test]
fn leaves_from_parts_are_found_and_whole_values_are_not() {
    let src = "\
for i in 0..3:
    print(i + (if i == 1:
        continue
    else:
        i
    ))
    let a = if i == 0: break else: i
    let b = match i:
        0 => continue
        _ => i
    f(x => if x: return 1 else: 2)
    while (if i > 1: break else: true):
        a
    match i:
        0 => continue
        _ => print(i)
    let v = x else:
        match x:
            None => break
";
    let file = crate::parse(src);
    assert!(file.diagnostics.is_empty(), "{:?}", file.diagnostics);
    let found: Vec<_> = leaves_from_parts(&file.root)
        .into_iter()
        .map(|(s, w)| (&src[s.start as usize..s.end as usize], w))
        .collect();
    assert_eq!(found, [("continue", "continue"), ("break", "break")]);
}
