//! The idiom lints newcomers hit, each from the corpus file or port that
//! showed it.

use super::*;
use crate::tests::*;

#[test]
fn chmax_if_with_a_comment_warns_without_a_fix() {
    let src = "let a = [1, 2]\nvar best = 0\nvar n = 0\nfor x in a:\n    n += 1\n    if x > best:  // keep the max\n        best = x\nprint(best, n)\n";
    let d = only(src);
    assert_eq!(d.code, crate::CHMIN_CHMAX_IF.code);
    assert!(d.fixes.is_empty());
}

#[test]
fn search_loops_newcomers_write_from_corpus_newcomer() {
    // n03: a flag with no `break` is still `all`.
    let flag = "let a = [1, 2]\nvar ok = true\nfor i in 0..1:\n    if a[i] > a[i + 1]:\n        ok = false\nprint(yes(ok))\n";
    assert!(
        fixed(flag).contains("let ok = (0..1).all(i => a[i] <= a[i + 1])"),
        "{}",
        fixed(flag)
    );
    // n45: a sentinel start keeps its value as the default.
    let sentinel = "let a = [1, 0]\nvar found = -1\nfor i in 0..2:\n    if a[i] == 0:\n        found = i\n        break\nprint(found)\n";
    assert!(fixed(sentinel).contains("let found = first(i for i in 0..2 if a[i] == 0) ?? -1"));
    assert!(only(sentinel)
        .help
        .as_deref()
        .unwrap()
        .contains("keep the `?Int`"));
    // n12: `and x == None` is a first match; a variable named `first` gets no fix.
    let guard = "let a = [1, 2]\nvar at: ?Int = None\nfor x in a:\n    if x % 2 == 0 and at == None:\n        at = x\nprint(at ?? -1)\n";
    assert!(
        fixed(guard).contains("let at: ?Int = first(x for x in a if x % 2 == 0)"),
        "{}",
        fixed(guard)
    );
    let named = guard.replace("at", "first");
    assert_eq!(only(&named).code.0, "L0501");
    assert!(only(&named).fixes.is_empty());
    for src in [flag, sentinel, guard, &named] {
        assert!(
            crate::fix_tests::checker_errors(&fixed(src)).is_empty(),
            "{}",
            fixed(src)
        );
    }
    // Setting it back to where it started is not a search.
    let same = "let a = [1, 2]\nvar ok = true\nfor x in a:\n    if x > 1:\n        ok = true\nprint(yes(ok))\n";
    assert!(taste_codes(same).is_empty());
}

#[test]
fn yes_no_printed_from_each_branch_is_yes() {
    let src = "let ok = true\nif ok:\n    print(\"YES\")\nelse:\n    print(\"NO\")\n";
    assert_eq!(
        only(src).help.as_deref(),
        Some("write `print(yes(ok))`; `YES` and `NO` are the default words")
    );
    let mixed = "let ok = true\nif ok:\n    print(\"YES\")\nelse:\n    \"NO\"\n";
    assert!(codes(mixed).iter().all(|c| *c != "L0524"));
}

#[test]
fn a_table_above_multitest_counted_into_leaks() {
    let src =
        "var cnt: [0..=9]Int = 0\n@!multitest\nlet x: 0..=9 = read()\ncnt[x] += 1\nprint(cnt[x])\n";
    assert_eq!(only(src).code.0, "L0211");
    let counter = "var t = 0\n@!multitest\nt += 1\nprint(t)\n";
    assert!(codes(counter).iter().all(|c| *c != "L0211"));
}

#[test]
fn plain_math_reads_latex_as_unicode() {
    let plain = crate::plain_math;
    assert_eq!(
        plain("costs \\(O(n \\log n)\\) here"),
        "costs O(n log n) here"
    );
    assert_eq!(
        plain("\\(O(n \\cdot \\alpha(n))\\), \\(O(\\sqrt{n})\\)"),
        "O(n · α(n)), O(√n)"
    );
    assert_eq!(plain("no math, `xs[0]`"), "no math, `xs[0]`");
}

#[test]
fn binom_from_a_factorial_and_an_inverse_factorial_table_is_flagged() {
    let mod_tables = "type Z = Mod<998244353>\nlet (a, b): (0..=1000, 0..=1000) = read()\nvar fact: List<Z> = [Z(1)]\nfor i in 1..=2000:\n    fact.push(fact[i - 1] * Z(i))\nvar inv_fact: List<Z> = []\nfor f in fact:\n    inv_fact.push(f.inv())\nprint(fact[a + b] * inv_fact[a] * inv_fact[b])\n";
    assert!(
        codes(mod_tables).contains(&"L0557"),
        "{:?}",
        codes(mod_tables)
    );
    let int_tables = "fn c(fact: List<Int>, ifact: List<Int>, n: Int, k: Int) -> Int =\n    fact[n] * ifact[k] % 998244353 * ifact[n - k] % 998244353\n";
    assert!(
        codes(int_tables).contains(&"L0557"),
        "{:?}",
        codes(int_tables)
    );
    let convolution =
        "fn f(xs: List<Int>, ys: List<Int>, i: Int, j: Int) -> Int = xs[i + j] * ys[i] * ys[j]\n";
    assert!(
        !codes(convolution).contains(&"L0557"),
        "{:?}",
        codes(convolution)
    );
}

#[test]
fn a_slice_copied_sorted_and_written_back_is_one_finding() {
    // corpus/newcomer/n33_copy_sort.cp: the push loop is part of the copy.
    let src = "let (n, l, r): (1..=100, 0..=99, 1..=100) = read()\nvar a: [0..n]Int = read()\nvar tmp: List<Int> = []\nfor i in l..r:\n    tmp.push(a[i])\ntmp.sort()\nfor i in l..r:\n    a[i] = tmp[i - l]\nprint(a)\n";
    assert_eq!(codes(src), ["L0522"]);
}

#[test]
fn a_prefix_sum_difference_is_not_a_binomial() {
    // corpus/newcomer/n51_run1_cf2285d.cp: `ps` is a running total.
    let src = "let n: 1..=10 = read()\nlet ps: [0..=n]Int = read()\nfn diff(x: 0..=n) -> Int = ps[x] - (ps[n] - ps[n - x])\nprint(diff(1))\n";
    assert!(!codes(src).contains(&"L0557"), "{:?}", codes(src));
}

#[test]
fn a_push_loop_that_walks_backwards_is_rev() {
    // corpus/newcomer/n47_reverse.cp and n74_string_reverse.cp.
    let arr = "let n: 1..=100 = read()\nlet a: [0..n]Int = read()\nvar b: List<Int> = []\nfor i in 0..n:\n    b.push(a[n - 1 - i])\nprint(b)\n";
    assert!(
        fixed(arr).contains("let b: List<Int> = a.rev.collect()\n"),
        "{}",
        fixed(arr)
    );
    let s = "let s: Str = read()\nvar t: List<Char> = []\nfor i in 0..s.len:\n    t.push(s[s.len - 1 - i])\nprint(Str(t))\n";
    assert!(
        fixed(s).contains("let t: List<Char> = s.rev.collect()\n"),
        "{}",
        fixed(s)
    );
    // Any other index is a plain comprehension.
    let other = "let n: 1..=100 = read()\nlet a: [0..n]Int = read()\nvar b: List<Int> = []\nfor i in 0..n - 1:\n    b.push(a[n - 2 - i])\nprint(b)\n";
    assert!(!fixed(other).contains("rev"), "{}", fixed(other));
}

#[test]
fn search_loop_help_is_the_exact_call_from_ports() {
    // corpus/problems/cco24p2: the help names the call the fix writes, with the test flipped.
    let flipped = "let a = [1, 2]\nlet b = [2, 1]\nvar same = true\nfor i in 0..2:\n    if a[i] != b[1 - i]: same = false\nprint(same)\n";
    let helps: Vec<String> = lint(flipped, &Config::default())
        .into_iter()
        .filter_map(|d| d.help)
        .collect();
    assert!(
        helps
            .iter()
            .any(|h| h.starts_with("write `(0..2).all(i => a[i] == b[1 - i])`")),
        "{helps:?}"
    );
    // A postfix receiver takes the method bare; `not not x` is `x`.
    let src = "let adj = [[1], [0]]\nlet vis = [true, false]\nvar has = false\nfor w in adj[0]:\n    if w == 1: has = true\nvar seen = true\nfor u in 0..2:\n    if not vis[u]: seen = false\nprint(has, seen)\n";
    let fixed = fixed(src);
    assert!(fixed.contains("adj[0].any(w => w == 1)"), "{fixed}");
    assert!(fixed.contains("(0..2).all(u => vis[u])"), "{fixed}");
}

#[test]
fn map_contains_then_index_binds_one_lookup() {
    let memo = "var memo = Map<Int, Int>()\n\
        fn f(x: Int) -> Int:\n\
        \x20   if x < 2: return x\n\
        \x20   if x in memo: return memo[x]\n\
        \x20   let r = f(x - 1) + f(x - 2)\n\
        \x20   memo[x] = r\n\
        \x20   r\n\
        print(f(read()))\n";
    assert_eq!(
        taste_codes(memo),
        ["L0554"],
        "`@memo` removes the lookup whole"
    );
    let lone = "var memo = Map<Int, Int>()\n\
        memo[1] = 1\n\
        let x: Int = read()\n\
        if x in memo: print(memo[x])\n";
    assert_eq!(taste_codes(lone), ["L0594"]);
    assert!(fixed(lone).ends_with("if let v = memo.get(x): print(v)\n"));

    let bound = "var pos = Map<Int, List<Int>>()\n\
        pos[1] = [0, 2]\n\
        let need: Int = read()\n\
        if need in pos:\n\
        \x20   let xs = pos[need]\n\
        \x20   print(xs.len)\n";
    assert_eq!(taste_codes(bound), ["L0594"]);
    assert!(fixed(bound).ends_with("if let xs = pos.get(need):\n    print(xs.len)\n"));

    let pushes = "var pos = Map<Int, List<Int>>()\n\
        pos[1] = []\n\
        let x: Int = read()\n\
        if x in pos:\n\
        \x20   pos[x].push(0)\n\
        \x20   print(pos[x].len)\n";
    let writes_in_fn = "var seen = Map<Int, Int>()\n\
        fn mark(x: Int):\n\
        \x20   seen[x] = 2\n\
        let x: Int = read()\n\
        seen[x] = 1\n\
        if x in seen:\n\
        \x20   mark(x)\n\
        \x20   print(seen[x])\n";
    let counter = "let c = Counter.of([5, 5])\n\
        let x: Int = read()\n\
        if x in c: print(c[x])\n";
    let other_key = "var m = Map<Int, Int>()\n\
        let (x, y): (Int, Int) = read()\n\
        if x in m: print(m[y])\n";
    for src in [pushes, writes_in_fn, counter, other_key] {
        assert!(!taste_codes(src).contains(&"L0594"), "{src}");
    }
}
