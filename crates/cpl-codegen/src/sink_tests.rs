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

#[test]
fn a_build_without_traps_sinks_a_consumer_that_divides() {
    let source = concat!(
        "let n: 1..=1000 = read()\n",
        "print(sum([i for i in 0..n if i % 2 == 0], of: x => 100 / x))\n",
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let target = crate::Target {
        checks: crate::Checks::None,
        ..crate::Target::default()
    };
    let cpp = crate::emit_for(&program, "t.cp", None, &target);
    assert!(cpp.contains("cpl_sink_"), "{cpp}");
}

#[test]
fn explain_names_a_sunk_reduction() {
    let source = concat!(
        "let n: 1..=1000 = read()\n",
        "print(sum(i * j for i in 0..n for j in i..n if (i + j) % 3 == 0))\n",
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let report = crate::explain::report("t.cp", source, &program);
    assert!(report.contains("[sunk] 2:7  sum(..)"), "{report}");
}

#[test]
fn a_u64_sum_with_a_default_of_builds_no_list_at_each_call() {
    let cpp = cpp(concat!(
        "let n: 1..=17 = read()\n",
        "domain V = 1..=n\n",
        "var adj: [V]Subset<V> = Subset<V>.empty\n",
        "var dp: [V]U64 = 1\n",
        "var ways: U64 = 1\n",
        "for mask in Subset<V>:\n",
        "    for v1 in mask:\n",
        "        ways *= sum(dp[v2] for v2 in adj[v1] & mask)\n",
        "print(ways, sum(dp[v] for v in V))\n",
    ));
    let solve = &cpp[cpp.find("void solve()").unwrap()..];
    assert!(
        !solve.contains("std::vector<unsigned long long>()"),
        "{solve}"
    );
    assert!(!solve.contains("push_back"), "{solve}");
}

#[test]
fn a_sum_whose_of_may_trap_keeps_the_list_only_at_that_call() {
    let cpp = cpp(concat!(
        "let n: 1..=1000 = read()\n",
        "print(sum([i for i in 0..n if i % 2 == 0], of: x => 100 / x))\n",
        "print(sum([i for i in 1..n], of: x => x * 2))\n",
    ));
    let solve = &cpp[cpp.find("void solve()").unwrap()..];
    assert_eq!(solve.matches("push_back").count(), 1, "{solve}");
    assert!(solve.contains("cpl_sink_"), "{solve}");
}

#[test]
fn a_reducer_that_hands_its_list_on_builds_no_list() {
    let cpp = cpp(concat!(
        "fn total(xs: List<Int>) -> Int = xs.fold((acc, x) => acc ^ x, 0)\n",
        "fn twice(xs: List<Int>) -> Int = total(xs)\n",
        "let n: 1..=1000 = read()\n",
        "print(twice([i * j for i in 0..n for j in i..n if (i + j) % 3 == 0]))\n",
    ));
    let solve = &cpp[cpp.find("void solve()").unwrap()..];
    assert!(solve.contains("cpl_sink_"), "{solve}");
    assert!(!solve.contains("push_back"), "{solve}");
    assert!(cpp.contains("(cpl_g"), "{cpp}");
}

const SUBSET_LOOP: &str = concat!(
    "domain B = 0..20\n",
    "let n: 1..=20 = read()\n",
    "var t = 0\n",
    "for k in 0..1000:\n",
    "    let s = Subset<B>.single(B(n % 20)).with(B(1))\n",
);

#[test]
fn a_reducer_over_a_yields_getter_runs_on_its_callback_form() {
    for reduce in [
        "    t += s.items.sum(of: b => b.ord * k)\n",
        "    t += s.items.fold((a, b) => a + b.ord, 0)\n",
    ] {
        let cpp = cpp(&format!("{SUBSET_LOOP}{reduce}print(t)\n"));
        let solve = &cpp[cpp.find("void solve()").unwrap()..];
        assert!(solve.contains("cpl_sink_"), "{solve}");
        assert!(solve.contains("cpl_each_Subset_"), "{solve}");
        assert!(!solve.contains("_items(s)"), "{solve}");
        assert!(!solve.contains("std::vector"), "{solve}");
    }
}

#[test]
fn a_closure_reading_a_var_keeps_the_list() {
    let cpp = cpp(&format!(
        "{SUBSET_LOOP}    var w = k\n    t += s.items.sum(of: b => b.ord * w)\n    w += 1\nprint(t)\n"
    ));
    let solve = &cpp[cpp.find("void solve()").unwrap()..];
    assert!(!solve.contains("cpl_sink_"), "{solve}");
}

const TAKE_EVENS: &str = concat!(
    "fn take_evens(mut xs: List<Int>) -> List<Int>:\n",
    "    var out: List<Int> = []\n",
    "    while let x = xs.pop():\n",
    "        if x % 2 == 0: out.push(x)\n",
    "    out\n",
    "let n: 1..=1000 = read()\n",
    "var xs = [i for i in 0..n]\n",
);

#[test]
fn the_len_of_a_list_a_fn_only_pushes_to_counts_with_no_list() {
    let cpp = cpp(&format!(
        "{TAKE_EVENS}print(take_evens(mut xs).len, xs.len)\n"
    ));
    let solve = &cpp[cpp.find("void solve()").unwrap()..];
    assert!(solve.contains("cpl_each_take_evens(xs, "), "{solve}");
    assert!(!solve.contains("take_evens(xs).size()"), "{solve}");
}

#[test]
fn a_loop_that_may_stop_early_runs_a_list_fn_to_its_end() {
    let cpp = cpp(&format!(
        "{TAKE_EVENS}for x in take_evens(mut xs): break\nprint(xs.len)\n"
    ));
    let solve = &cpp[cpp.find("void solve()").unwrap()..];
    assert!(!solve.contains("cpl_each_take_evens"), "{solve}");
}

#[test]
fn the_len_of_a_list_a_fn_starts_with_values_keeps_the_list() {
    let cpp = cpp(concat!(
        "fn halves(n: Int) -> List<Int>:\n",
        "    var out = [i for i in 0..n]\n",
        "    out.push(n)\n",
        "    out\n",
        "let n: 1..=1000 = read()\n",
        "print(halves(n).len)\n",
    ));
    assert!(!cpp.contains("cpl_each_halves"), "{cpp}");
}
