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

#[test]
fn a_running_total_used_whole_keeps_its_loop() {
    // corpus/newcomer/n52_run1_cf2289d.cp (acceptance run 3, `prefix-sum-manual-push`):
    // a `Prefix` has no `any`, so `Prefix(a)` would not compile there.
    let whole = "let n: 1..=10 = read()\nlet a: [0..n]Int = read()\nvar pre: List<Int> = []\nvar s = 0\nfor x in a:\n    s += x\n    pre.push(s)\nprint(yes(pre.any(t => t < 0)))\n";
    assert!(!codes(whole).contains(&"L0521"), "{:?}", codes(whole));
    let indexed = whole.replace(
        "print(yes(pre.any(t => t < 0)))",
        "print(pre[n - 1] - pre[0])",
    );
    assert!(codes(&indexed).contains(&"L0521"), "{:?}", codes(&indexed));
}

#[test]
fn a_key_guard_then_index_is_let_else() {
    // Acceptance run 3, `map-get-index-workaround`: `if key not in pos: return 0`, then `pos[key]` three times.
    let ret = "fn f(pos: Map<Int, Int>, key: Int) -> Int:\n    if key not in pos: return 0\n    pos[key] * pos[key] + pos[key]\nprint(f(Map(), 1))\n";
    assert!(codes(ret).contains(&"L0594"), "{:?}", codes(ret));
    let out = fixed(ret);
    assert!(
        out.contains("    let v = pos.get(key) else: return 0\n    v * v + v\n"),
        "{out}"
    );
    assert!(crate::fix_tests::checker_errors(&out).is_empty(), "{out}");
    let cont = "let m: Map<Int, Int> = Map()\nfor k in 0..5:\n    if k not in m:\n        continue\n    let x = m[k]\n    print(x)\n";
    let out = fixed(cont);
    assert!(
        out.contains("    let x = m.get(k) else: continue\n    print(x)\n"),
        "{out}"
    );
    assert!(crate::fix_tests::checker_errors(&out).is_empty(), "{out}");
    let write = "var m: Map<Int, Int> = Map()\nfn f(k: Int) -> Int:\n    if k not in m: return 0\n    m[k] += 1\n    m[k]\nprint(f(1))\n";
    assert!(!codes(write).contains(&"L0594"), "{:?}", codes(write));
    let other = "fn f(m: Map<Int, Int>, k: Int) -> Int:\n    if k not in m: return 0\n    m[k + 1]\nprint(f(Map(), 1))\n";
    assert!(!codes(other).contains(&"L0594"), "{:?}", codes(other));
}

#[test]
fn a_stand_in_test_then_unwrap_is_a_let_header() {
    // Acceptance run 3, n54: `while (sets[id[c]].last ?? 0) > cap:` then `let w = sets[id[c]].last!`.
    let src = "var s: List<Int> = [1, 5, 9]\nlet cap = 4\nvar n = 0\nwhile (s.last ?? 0) > cap:\n    let w = s.last!\n    n += w\n    s = [x for x in s if x != w]\nprint(n)\n";
    assert!(codes(src).contains(&"L0595"), "{:?}", codes(src));
    let out = fixed(src);
    assert!(
        out.contains("while let w = s.last, w > cap:\n    n += w\n"),
        "{out}"
    );
    assert!(crate::fix_tests::checker_errors(&out).is_empty(), "{out}");
    let elif = "let a = [3, 8]\nlet best = a.find(x => x > 5)\nif a.len == 0:\n    print(0)\nelif (best ?? 0) >= 6:\n    let b = best!\n    print(b)\n";
    let out = fixed(elif);
    assert!(
        out.contains("elif let b = best, b >= 6:\n    print(b)\n"),
        "{out}"
    );
    assert!(crate::fix_tests::checker_errors(&out).is_empty(), "{out}");
    // A call may answer differently when it runs again.
    let call = "var q = [1, 2]\nwhile (q.pop() ?? 0) > 0:\n    let w = q.pop()!\n    print(w)\n";
    assert!(!codes(call).contains(&"L0595"), "{:?}", codes(call));
    let tuple = "let a = [(1, 2)]\nlet p = a.find(t => t.0 > 0)\nif (p ?? (0, 0)) > (0, 1):\n    let (x, y) = p!\n    print(x + y)\n";
    assert!(!codes(tuple).contains(&"L0595"), "{:?}", codes(tuple));
}

#[test]
fn a_found_flag_beside_its_value_is_an_optional() {
    // corpus/newcomer/n59_run1_cf2272d.cp: `found` only says `hub_tree` is set.
    let src = "let a = [3, 4]\nvar found = false\nvar last = 0\nfor x in a:\n    if x % 2 == 0:\n        last = x\n        found = true\nif found: print(last)\n";
    assert_eq!(codes(src), ["L0596"]);
    let pair = "let a = [3, 4]\nvar seen = false\nvar num = 0\nvar den = 1\nfor x in a:\n    if not seen or x * den > num:\n        seen = true\n        num = x\n        den = 1\nif seen: print(num, den)\n";
    let d = only(pair);
    assert!(d.message.contains("`num`, `den` have"), "{}", d.message);
}

#[test]
fn a_flag_that_says_more_than_a_value_is_set_stays_quiet() {
    // std/graph.cp boruvka: the value changes by `-=`, so the flag counts merges.
    let counted = "let a = [3, 4]\nvar left = 5\nvar merged = false\nfor x in a:\n    if x > 3:\n        left -= 1\n        merged = true\nprint(yes(merged), left)\n";
    // corpus/problems/sac22cc1p3: the value lives in a narrower scope.
    let narrower = "let a = [3, 4]\nvar changed = false\nfor x in a:\n    var at = 0\n    if x > 3:\n        at = x\n        changed = true\n    print(at)\nprint(yes(changed))\n";
    // std/bag.cp: the value starts as real data, not a placeholder.
    let data = "var a = [3, 4]\nvar small = a.clone()\nvar flip = false\nif a.len > 1:\n    small = [1]\n    flip = true\nprint(yes(flip), small)\n";
    // The flag is cleared again.
    let cleared = "let a = [3, 4]\nvar open = false\nvar last = 0\nfor x in a:\n    if x > 3:\n        last = x\n        open = true\n    else:\n        open = false\nprint(yes(open), last)\n";
    for src in [counted, narrower, data, cleared] {
        assert!(!codes(src).contains(&"L0596"), "{src}\n{:?}", codes(src));
    }
}

#[test]
fn a_counting_loop_over_one_sequence_is_its_count_method() {
    // Decision 53; BLOCKERS `count-two-spellings`: the lint taught
    // `count(x > 10 for x in a)` beside docs teaching `a.count(x => x > 10)`.
    let src = "let a: List<Int> = [3, 12, 40]\nvar big = 0\nfor x in a:\n    if x > 10:\n        big += 1\nprint(big)\n";
    let d = lint_one(src, "L0504");
    assert!(
        d.help
            .as_deref()
            .unwrap()
            .contains("`a.count(x => x > 10)`"),
        "{:?}",
        d.help
    );
    assert!(d.labels.is_empty(), "{:?}", d.labels);
    assert!(crate::apply_fixes(src, &[d])
        .0
        .contains("big = a.count(x => x > 10)\nprint"));
}

#[test]
fn a_counting_loop_over_a_map_keeps_the_generator() {
    let src = "let m: Map<Int, Int> = [1: 2, 30: 4]\nvar big = 0\nfor kv in m:\n    if kv.0 > 10:\n        big += 1\nprint(big)\n";
    let d = lint_one(src, "L0504");
    assert!(
        d.help
            .as_deref()
            .unwrap()
            .contains("`count(kv.0 > 10 for kv in m)`"),
        "{:?}",
        d.help
    );
    assert!(crate::apply_fixes(src, &[d])
        .0
        .contains("big = count(kv.0 > 10 for kv in m)\nprint"));
}

#[test]
fn a_counting_loop_over_a_range_counts_the_range() {
    let src = "let n = 10\nvar even = 0\nfor i in 0..n:\n    if i % 2 == 0:\n        even += 1\nprint(even)\n";
    let d = lint_one(src, "L0504");
    assert!(
        d.help
            .as_deref()
            .unwrap()
            .contains("`(0..n).count(i => i % 2 == 0)`"),
        "{:?}",
        d.help
    );
}

fn lint_one(src: &str, code: &str) -> Diagnostic {
    let diags = lint(src, &Config::default());
    let mut found: Vec<_> = diags.into_iter().filter(|d| d.code.0 == code).collect();
    assert_eq!(found.len(), 1, "{found:?}");
    found.remove(0)
}

#[test]
fn a_counting_test_that_writes_keeps_the_generator() {
    // crates/cpl-cli/tests/programs/s2_keep_bool.cp: a lambda can't write `hi`.
    let src = "var hi = 0\nvar grew = 0\nfor x in [3, 1, 4]:\n    if hi max= x: grew += 1\nprint(hi, grew)\n";
    let d = lint_one(src, "L0504");
    assert!(
        d.help
            .as_deref()
            .unwrap()
            .contains("`count(hi max= x for x in [3, 1, 4])`"),
        "{:?}",
        d.help
    );
}

#[test]
fn a_counting_loop_over_a_product_domain_keeps_the_generator() {
    // spec/conformance/core-04/product-alias-walks.cp: a product domain is no `Seq`.
    let src = "domain R = 0..2\ndomain C = 0..2\ntype Cell = (R, C)\nlet grid: [Cell]Char = '.'\nvar n = 0\nfor p in Cell:\n    if grid[p] == '.':\n        n += 1\nprint(n)\n";
    let d = lint_one(src, "L0504");
    assert!(
        d.help
            .as_deref()
            .unwrap()
            .contains("`count(grid[p] == '.' for p in Cell)`"),
        "{:?}",
        d.help
    );
}

#[test]
fn a_list_only_the_checker_types_is_still_an_unproven_index() {
    let src = "fn rows(n: Int) -> List<Int>:\n    return [n].filled(0)\nfn word() -> Str:\n    return \"abcde\"\nlet xs = rows(5)\nlet w = word()\nvar t = 0\nfor i in 0..5:\n    for j in 0..5:\n        t += xs[(i + j) % 5]\n        if w[(i + j) % 5] == 'a': t += 1\nprint(t)\n";
    let mut config = Config::default();
    config.set(&crate::UNPROVEN_LIST_INDEX, crate::Level::Warn);
    let diags = lint(src, &config);
    let found: Vec<_> = diags
        .iter()
        .filter(|d| d.code == crate::UNPROVEN_LIST_INDEX.code)
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].message.contains("xs["), "{}", found[0].message);
}
