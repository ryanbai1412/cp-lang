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
