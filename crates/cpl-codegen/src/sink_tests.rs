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

#[test]
fn a_fold_whose_op_calls_trap_free_fns_builds_no_list() {
    let cpp = cpp(concat!(
        "type Z = Mod<998_244_353>\n",
        "let n: 1..=1000 = read()\n",
        "print(fold(*, Z(i) for i in 1..=n if i % 3 != 0))\n",
    ));
    assert!(cpp.contains("cpl_sink_"), "{cpp}");
}

#[test]
fn a_mod_sum_over_a_generator_builds_no_list() {
    let cpp = cpp(concat!(
        "type Z = Mod<1_000_000_007>\n",
        "let n: 1..=1000 = read()\n",
        "print(sum(Z(i) * Z(i + 1) for i in 1..=n))\n",
    ));
    assert!(cpp.contains("cpl_sink_"), "{cpp}");
    assert!(!cpp.contains("check_range"), "{cpp}");
}

#[test]
fn a_consumer_calling_a_fn_that_may_trap_keeps_the_list() {
    let consumer = |body: &str| {
        cpp(&format!(
            "fn each(x: Int) -> Int = {body}\n\
             fn total(xs: List<Int>) -> Int:\n    var acc = 0\n    for x in xs: acc += each(x)\n    acc\n\
             let n: 1..=1000 = read()\nprint(total([i for i in 0..n]))\n"
        ))
    };
    assert!(consumer("x * 2 + 1").contains("cpl_sink_"));
    let cpp = consumer("100 / x");
    assert!(!cpp.contains("cpl_sink_"), "{cpp}");
}
