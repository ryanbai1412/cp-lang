//! One-item groups, multi-clause loops and `is` conditions parse to their
//! shapes.

use crate::ast::*;
use crate::*;

fn stmts(src: &str) -> Vec<Stmt> {
    let file = parse_file(src);
    assert!(
        file.diagnostics.is_empty(),
        "{src:?}: {:#?}",
        file.diagnostics
    );
    file.items
        .into_iter()
        .flat_map(|(_, item)| item.stmts)
        .collect()
}

fn let_parts(stmt: &Stmt) -> (&Pattern, Option<&Type>, Option<&Expr>) {
    let StmtKind::Let { pat, ty, init, .. } = &stmt.kind else {
        panic!("not a let: {stmt:?}")
    };
    (pat, ty.as_ref(), init.as_ref())
}

#[test]
fn a_parenthesized_pattern_or_type_of_one_is_that_one() {
    let s = stmts("let (a): (Int) = 1\nlet (b, c): (Int, Int) = (1, 2)\n");
    let (pat, ty, _) = let_parts(&s[0]);
    assert!(
        matches!(&pat.kind, PatternKind::Name(n) if n == "a"),
        "{pat:?}"
    );
    assert!(
        !matches!(ty.map(|t| &t.kind), Some(TypeKind::Tuple(_))),
        "{ty:?}"
    );
    let (pat, ty, _) = let_parts(&s[1]);
    assert!(
        matches!(&pat.kind, PatternKind::Tuple(p) if p.len() == 2),
        "{pat:?}"
    );
    assert!(
        matches!(ty.map(|t| &t.kind), Some(TypeKind::Tuple(t)) if t.len() == 2),
        "{ty:?}"
    );
}

#[test]
fn a_list_of_one_before_a_value_is_a_fill_and_alone_is_a_list() {
    let s = stmts("let a = [n] 0\nlet b = [n]\nlet c = [n, m]\n");
    let init = |i: usize| &let_parts(&s[i]).2.expect("an initializer").kind;
    assert!(
        matches!(init(0), ExprKind::TableFill { .. }),
        "{:?}",
        init(0)
    );
    assert!(
        matches!(init(1), ExprKind::List(xs) if xs.len() == 1),
        "{:?}",
        init(1)
    );
    assert!(
        matches!(init(2), ExprKind::List(xs) if xs.len() == 2),
        "{:?}",
        init(2)
    );
}

#[test]
fn a_counted_domain_of_one_is_a_range_and_of_more_stays_a_list() {
    let file = parse_file("domain E = [m]\ndomain F = [a, b]\n");
    let shown = format!("{:?}", file.items);
    assert_eq!(shown.matches("Range {").count(), 1, "{shown}");
    assert_eq!(shown.matches("List(").count(), 1, "{shown}");
}

#[test]
fn shared_names_each_get_the_value_and_one_name_is_one_let() {
    let s = stmts("var a, b, c: Int = 0\nvar d: Int = 0\n");
    assert_eq!(s.len(), 4, "{s:#?}");
    for (stmt, name) in s.iter().zip(["a", "b", "c", "d"]) {
        let (pat, ty, init) = let_parts(stmt);
        assert!(
            matches!(&pat.kind, PatternKind::Name(n) if n == name),
            "{pat:?}"
        );
        assert!(ty.is_some() && init.is_some(), "{stmt:?}");
    }
}

#[test]
fn a_multi_clause_for_nests_outer_first_with_its_filter_and_else_outside() {
    let src = "for i in 0..n if i > 0, j in 0..m if j < i:\n    print(i)\nelse:\n    print(0)\n";
    let s = stmts(src);
    let StmtKind::For {
        pat,
        body,
        else_body,
        ..
    } = &s[0].kind
    else {
        panic!("not a for: {s:?}")
    };
    assert!(matches!(&pat.kind, PatternKind::Name(n) if n == "i"));
    assert!(else_body.is_some(), "the else belongs to the outer loop");
    let StmtKind::Expr(Expr {
        kind: ExprKind::If { branches, .. },
        ..
    }) = &body.stmts[0].kind
    else {
        panic!("the outer filter is an if: {body:?}")
    };
    let StmtKind::For {
        pat,
        body,
        else_body,
        ..
    } = &branches[0].1.stmts[0].kind
    else {
        panic!("the inner loop is inside the outer filter: {branches:?}")
    };
    assert!(matches!(&pat.kind, PatternKind::Name(n) if n == "j"));
    assert!(else_body.is_none());
    assert!(matches!(
        &body.stmts[0].kind,
        StmtKind::Expr(Expr {
            kind: ExprKind::If { .. },
            ..
        })
    ));
    let full = src.find("else").expect("an else");
    let filter = &branches[0].1.stmts[0];
    assert!(
        (filter.span.end as usize) < full,
        "the inner filter's span stops before the else: {filter:?}"
    );
}

#[test]
fn an_is_condition_is_a_let_and_a_written_if_else_false_stays_an_expression() {
    let cond = |src: &str| {
        let s = stmts(src);
        let StmtKind::While { cond, .. } = &s[0].kind else {
            panic!("not a while: {s:?}")
        };
        cond.clone()
    };
    let is = cond("while x is Some(y) and y > 0:\n    f(y)\n");
    assert!(
        matches!(&is, Cond::Let { and: Some(rest), .. } if matches!(**rest, Cond::Expr(_))),
        "{is:?}"
    );
    let plain = cond("while x is Some(_):\n    f(x)\n");
    assert!(matches!(&plain, Cond::Let { and: None, .. }), "{plain:?}");
    let written = cond("while (if let Some(y) = x: y > 0 else: false):\n    f(x)\n");
    assert!(matches!(&written, Cond::Expr(_)), "{written:?}");
}

#[test]
fn a_coalesce_mixed_with_arithmetic_says_both_readings() {
    let diags = parse_file("let v = a ?? b + c\n").diagnostics;
    assert_eq!(diags.len(), 1, "{diags:#?}");
    let help = format!("{:?}", diags[0]);
    assert!(
        help.contains("`(a ?? b) + c`") && help.contains("`a ?? (b + c)`"),
        "{help}"
    );
}
