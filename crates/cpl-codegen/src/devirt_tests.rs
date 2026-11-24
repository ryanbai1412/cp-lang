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
fn local_bound_to_a_made_fn_field_calls_its_one_closure() {
    let out = code(concat!(
        "fn fold(xs: List<Int>) -> Int:\n",
        "    let mon = Monoid<Int>.of(+)\n",
        "    let f = mon.op\n",
        "    var a = mon.id\n",
        "    for x in xs: a = f(a, x)\n",
        "    a\n",
        "print(fold([3, 1, 2]))\n",
    ));
    assert!(
        !out.contains("f(a, x)") && out.contains("a = cpl_fn"),
        "{out}"
    );
}

#[test]
fn local_bound_to_a_rewritten_struct_field_keeps_its_fn_value_call() {
    let out = code(concat!(
        "fn fold(xs: List<Int>) -> Int:\n",
        "    var mon = Monoid<Int>.of(min)\n",
        "    if xs.len > 2: mon = Monoid<Int>.of(+)\n",
        "    let f = mon.op\n",
        "    var a = mon.id\n",
        "    for x in xs: a = f(a, x)\n",
        "    a\n",
        "print(fold([3, 1, 2]))\n",
    ));
    assert!(out.contains("f(a, x)"), "{out}");
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

#[test]
fn a_fn_named_twice_as_one_field_value_is_still_one_fn() {
    let out = code(concat!(
        "fn low(a: Int, b: Int) -> Int = min(a, b)\n",
        "let m = Monoid<Int>(id: 9, op: low)\n",
        "let k = Monoid<Int>(id: 8, op: low)\n",
        "print(m.op(3, m.id) + k.op(1, k.id))\n",
    ));
    assert!(!out.contains("std::function"), "{out}");
}

#[test]
fn tuple_of_lambdas_binds_each_without_std_function() {
    let out = cpp("fn f(n: Int) -> Int:
    let (pre, suf) = ((i: Int) => n + i, (i: Int) => 2 * n + i)
    pre(1) + suf(2)
print(f(read()))
");
    assert!(!out.contains("std::function"), "{out}");
}

/// The struct `make` returns without `b`.
const TWO: &str = concat!(
    "struct Two:\n",
    "    pub a: List<Int>\n",
    "    pub b: List<Int>\n",
    "fn make(n: Int) -> Two:\n",
    "    var a = List<Int>()\n",
    "    var b = List<Int>()\n",
    "    for i in 0..n:\n",
    "        a.push(i)\n",
    "        b.push(i * i)\n",
    "    Two(a:, b:)\n",
    "let t = make(read())\n",
    "print(t.a.len)\n",
);

#[test]
fn a_list_only_a_dropped_field_holds_is_not_built() {
    let out = code(TWO);
    assert!(
        out.contains("struct Two {\n  std::vector<ll> a;\n};"),
        "{out}"
    );
    assert!(!out.contains("std::vector<ll> b"), "{out}");
    assert!(
        !out.contains("b.push_back") && !out.contains("b.reserve"),
        "{out}"
    );
    // `i * i` may overflow, so its check still runs where the push was.
    assert!(out.contains("(void)(cpl::imul(i, i));"), "{out}");
}

#[test]
fn a_dropped_push_whose_value_cannot_trap_leaves_nothing() {
    let out = code(&TWO.replace("n: Int", "n: 0..=1000"));
    assert!(!out.contains("std::vector<ll> b"), "{out}");
    assert!(!out.contains("i * i") && !out.contains("imul"), "{out}");
}

#[test]
fn a_dropped_local_that_reads_itself_goes_unless_its_value_may_trap() {
    let out = code(concat!(
        "struct R:\n",
        "    pub a: List<Int>\n",
        "    pub top: List<Int>\n",
        "    pub far: List<Int>\n",
        "fn make(n: 1..=100, w: Int) -> R:\n",
        "    domain V = 0..n\n",
        "    var a: List<Int> = []\n",
        "    var top: [V]V = [V] v => v\n",
        "    var far: [V]Int = 0\n",
        "    let u = V.first!\n",
        "    for v in V:\n",
        "        a.push(v.ord)\n",
        "        top[v] = top[u]\n",
        "        far[v] = far[u] + w\n",
        "    R(a:, top: [], far: [])\n",
        "let r = make(read(), read())\n",
        "print(r.a.len)\n",
    ));
    assert!(
        !out.contains("top[") && !out.contains("& top"),
        "top only reads itself: {out}"
    );
    assert!(
        out.contains("= cpl::iadd(far[u], w);"),
        "the sum may overflow: {out}"
    );
}
