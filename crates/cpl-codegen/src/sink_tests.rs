//! A fn handed a list built on the spot runs its loop at each push (`crate::sink`).

use crate::tests::cpp;

#[test]
fn a_sum_over_a_generator_builds_no_list() {
    let cpp = cpp(concat!(
        "let n: 1..=1000 = read()\n",
        "print(sum(i * j for i in 0..n for j in i..n if (i + j) % 3 == 0))\n",
    ));
    assert!(cpp.contains("cpl_sink_"), "{cpp}");
    let solve = &cpp[cpp.find("void solve()").unwrap()..];
    assert!(!solve.contains("push_back"), "{solve}");
}

#[test]
fn a_consumer_that_may_trap_keeps_the_list() {
    let cpp = cpp(concat!(
        "let n: 1..=1000 = read()\n",
        "print(sum([i for i in 0..n if i % 2 == 0], of: x => 100 / x))\n",
    ));
    assert!(!cpp.contains("cpl_sink_"), "{cpp}");
}
