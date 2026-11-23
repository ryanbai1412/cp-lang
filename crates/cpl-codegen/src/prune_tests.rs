//! A branch decided at compile time is left out with what only it calls (`crate::prune`).

use crate::tests::cpp;

#[test]
fn a_branch_on_an_op_value_keeps_only_the_one_taken() {
    let cpp = cpp(concat!(
        "fn same<A, B>(a: A, b: B) -> Bool = a == b\n",
        "fn other_way(a: Int, b: Int) -> Int = a * b + 1\n",
        "fn combine<F>(a: Int, b: Int, on: F = +) -> Int:\n",
        "    if not same(on, +):\n",
        "        return other_way(a, b)\n",
        "    a + b\n",
        "let n: 1..=10 = read()\n",
        "print(combine(n, n, on: +))\n",
    ));
    let code: String = cpp.lines().filter(|l| !l.starts_with("//")).collect();
    assert!(!code.contains("other_way"), "{cpp}");
    assert!(
        !code.contains("if (false)") && !code.contains("if (!(true))"),
        "{cpp}"
    );
}

#[test]
fn the_start_of_a_range_from_a_literal_is_the_literal() {
    let cpp = cpp(concat!(
        "let n: 1..=10 = read()\n",
        "let xs = [i for i in 0..n]\n",
        "print((0..xs.len).start)\n",
        "print((0..n * 2).start)\n",
    ));
    let solve = &cpp[cpp.find("void solve").unwrap()..];
    assert!(solve.contains("put_int(0)"), "{cpp}");
    assert_eq!(solve.matches("::make(").count(), 1, "{cpp}");
}

#[test]
fn a_fn_only_an_assert_calls_is_left_out_of_a_build_without_checks() {
    let source = concat!(
        "fn is_even(x: Int) -> Bool = x % 2 == 0\n",
        "let n: 1..=10 = read()\n",
        "assert is_even(n)\n",
        "print(n)\n",
    );
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let build = |checks| {
        let target = crate::Target {
            checks,
            ..crate::Target::default()
        };
        crate::emit_for(&program, "t.cp", None, &target)
    };
    assert!(build(crate::Checks::Traps).contains("is_even("));
    let none = build(crate::Checks::None);
    assert!(!none.contains("is_even("), "{none}");
}

#[test]
fn a_runtime_fn_named_only_by_a_comment_or_a_member_is_unused() {
    let file = concat!(
        "namespace cpl {\n",
        "// `first` of a view: its first element\n",
        "template <class T> T first(T v) { return v; }\n",
        "}  // namespace cpl\n",
        "int main() { auto p = std::pair<int, int>(); return p.first; }\n",
    );
    assert_eq!(
        crate::runtime::unused_lines(file),
        ["template <class T> T first(T v) { return v; }"]
    );
}

#[test]
fn a_comparison_of_two_literals_keeps_only_the_branch_taken() {
    let cpp = cpp(concat!(
        "type Z = Mod<998_244_353>\n",
        "let a: 0..=10 = read()\n",
        "print(Z(a) * Z(a))\n",
    ));
    assert!(!cpp.contains("998244353 < 3037000500"), "{cpp}");
    assert!(!cpp.contains("mul_mod("), "{cpp}");
}

#[test]
fn an_optional_comparator_nobody_passes_is_never_tested() {
    let cpp = cpp(concat!(
        "var h = MinHeap<Int>()\n",
        "let n: 1..=10 = read()\n",
        "for i in 0..n: h.push(n - i)\n",
        "print(h.pop()!)\n",
    ));
    let start = cpp.find("_less(").expect("a less fn");
    let less = &cpp[start..start + cpp[start..].find("\n}").expect("its end")];
    assert!(!less.contains("has_value()"), "{less}");
    let start = cpp.find("_new_").expect("a new fn");
    let new = &cpp[start..start + cpp[start..].find("\n}").expect("its end")];
    assert!(!new.contains("has_value()"), "{new}");
}

#[test]
fn an_optional_comparator_some_heap_gets_is_still_tested() {
    let cpp = cpp(concat!(
        "var h = MinHeap<Int>()\n",
        "var g: Heap<Int> = Heap(by: x => -x)\n",
        "let n: 1..=10 = read()\n",
        "for i in 0..n:\n",
        "    h.push(n - i)\n",
        "    g.push(i)\n",
        "print(h.pop()!, g.pop()!)\n",
    ));
    let start = cpp.find("_less(").expect("a less fn");
    let less = &cpp[start..start + cpp[start..].find("\n}").expect("its end")];
    assert!(less.contains("has_value()"), "{less}");
}
