//! Calls through a fn value the devirt pass resolves to one closure.

use crate::tests::cpp;

/// The emitted C++ without the comment that quotes the source.
fn code(source: &str) -> String {
    cpp(source)
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn local_from_struct_maker_calls_its_one_closure() {
    let out = code(concat!(
        "fn fold(xs: List<Int>) -> Int:\n",
        "    let lo = Monoid<Int>.of(min)\n",
        "    let s = Monoid<Int>.of(+)\n",
        "    var a = lo.id\n",
        "    var b = s.id\n",
        "    for x in xs:\n",
        "        a = lo.op(a, x)\n",
        "        b = s.op(b, x)\n",
        "    a + b\n",
        "print(fold([3, 1, 2]))\n",
    ));
    assert!(!out.contains("lo.op(") && !out.contains("s.op("), "{out}");
    assert!(
        out.contains("a = cpl_fn") && out.contains("b = cpl_fn"),
        "{out}"
    );
}

#[test]
fn reassigned_struct_local_keeps_its_fn_value_call() {
    let out = code(concat!(
        "fn fold(xs: List<Int>) -> Int:\n",
        "    var m = Monoid<Int>.of(min)\n",
        "    if xs.len > 2: m = Monoid<Int>.of(+)\n",
        "    var a = m.id\n",
        "    for x in xs: a = m.op(a, x)\n",
        "    a\n",
        "print(fold([3, 1, 2]))\n",
    ));
    assert!(out.contains("m.op(a, x)"), "{out}");
}
