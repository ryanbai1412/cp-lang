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

#[test]
fn newcomer_fixes_still_check() {
    // n07: `var lo` and `var hi` are two top-level items; `hi`'s text was read at `lo`'s offset.
    let search = "let k = 10\nvar lo = 1\nvar hi = 1000000001\nwhile lo < hi:\n    let mid = (lo + hi) / 2\n    if mid * 2 < k:\n        lo = mid + 1\n    else:\n        hi = mid\nprint(lo)\n";
    let out = fixed(search);
    assert!(
        out.contains("(1..1000000001).first_true(mid => mid * 2 >= k)"),
        "{out}"
    );
    // n25: `all`/`any` take a predicate, not a generator. (n25's own mirrored
    // compare is `palindrome_by_hand`, which supersedes the search.)
    let palindrome = "let s: Str = \"abba\"\nlet n = s.len\nvar ok = true\nfor i in 0..n / 2:\n    if s[i] != s[n - 1 - i]:\n        ok = false\n        break\nprint(yes(ok))\n";
    let found = codes(palindrome);
    assert!(
        found.contains(&"L0583") && !found.contains(&"L0501"),
        "{found:?}"
    );
    let all = "let s: Str = \"abba\"\nlet n = s.len\nvar ok = true\nfor i in 0..n / 2:\n    if s[i] != s[i + 1]:\n        ok = false\n        break\nprint(yes(ok))\n";
    let any = "let a = [1, 2]\nvar ok = false\nfor x in a:\n    if x > 1:\n        ok = true\n        break\nprint(yes(ok))\n";
    for src in [search, all, any] {
        let out = fixed(src);
        assert!(crate::fix_tests::checker_errors(&out).is_empty(), "{out}");
    }
    assert!(fixed(all).contains("(0..n / 2).all(i => s[i] == s[i + 1])"));
    assert!(fixed(any).contains("let ok = a.any(x => x > 1)"));
}

#[test]
fn library_fn_is_not_told_to_call_itself() {
    // std/math.cp: `divisors` is where std writes the trial division L0561 teaches.
    let body = "fn divisors(x: Int) -> List<Int>:\n    var (small, large) = (List<Int>(), List<Int>())\n    var d = 1\n    while d * d <= x:\n        if x % d == 0:\n            small.push(d)\n            if d * d != x: large.push(x / d)\n        d += 1\n    small + large.rev\n";
    assert!(!codes(&format!("pub {body}")).contains(&"L0561"));
    // Outside a library module the lint still teaches: std's `divisors` replaces it.
    assert!(codes(&format!("{body}print(divisors(12))\n")).contains(&"L0561"));
    // The fn a help names is not enough when it is another one.
    let other = body.replacen("fn divisors", "fn my_divs", 1);
    assert!(codes(&format!("pub {other}")).contains(&"L0561"));
}

#[test]
fn trial_division_under_an_if_factors() {
    // std/math.cp `factor`: the dividing loop sits under `if rest % p == 0`.
    let src = "fn f(x: Int) -> List<(Int, Int)>:\n    var out = List<(Int, Int)>()\n    var (rest, p) = (x, 2)\n    while p * p <= rest:\n        if rest % p == 0:\n            var e = 0\n            while rest % p == 0:\n                rest /= p\n                e += 1\n            out.push((p, e))\n        p += 1\n    if rest > 1: out.push((rest, 1))\n    out\nprint(f(12))\n";
    let d = lint(src, &all_on());
    let d = d.iter().find(|d| d.code.0 == "L0561").expect("L0561");
    assert!(
        d.help.as_deref().unwrap().contains("`factor(rest)`"),
        "{d:#?}"
    );
}

#[test]
fn copy_comprehension_needs_a_seq() {
    // std/bag.cp: a `SortedSet` walks in `for` but has no `collect`.
    let set = "var s = SortedSet<Int>.of([1, 3])\nassert [x for x in s] == [1, 3]\n";
    assert!(!codes(set).contains(&"L0549"));
    let seqs = "domain V = 0..3\nlet a = [3, 1]\nprint([x for x in a], [v for v in V], [c for c in \"ab\"], [i for i in 0..3])\n";
    assert_eq!(codes(seqs).iter().filter(|c| **c == "L0549").count(), 4);
}

#[test]
fn split_range_check_fixes_to_a_chain() {
    // corpus noi14p5 and ioi19p6: the two comparisons become one chain.
    let src = "let (x, n): (Int, 1..=100) = read()\nif x < n and x >= 0:\n    print(1)\nif not (0 < x and x < n):\n    print(2)\n";
    let out = fixed(src);
    assert!(out.contains("if 0 <= x < n:"), "{out}");
    assert!(out.contains("if not (0 < x < n):"), "{out}");
    // std/lazy.cp and std/marks.cp: a longer chain, and a chain inside a
    // longer `and`, written in the statement before (spans are per item).
    let long = "let (x, n): (Int, 1..=100) = read()\nlet y: Int = read()\nassert 0 <= x and x <= y and y <= n\nif x >= 0 and x < n and n > 5:\n    print(1)\n";
    let out = fixed(long);
    assert!(out.contains("assert 0 <= x <= y <= n\n"), "{out}");
    assert!(out.contains("if 0 <= x < n and n > 5:"), "{out}");
    // A bound call that would run in a new order keeps only the help.
    let kept =
        "let (x, n): (Int, 1..=100) = read()\nif x < n.min(9) and x >= abs(n):\n    print(2)\n";
    assert_eq!(fixed(kept), kept);
    assert!(codes(kept).contains(&"L0551"));
}

#[test]
fn copy_comprehension_fixes_only_a_list_copy() {
    let src = "let a = [3, 1]\nvar b = [x for x in a]\nb.push(0)\nprint(b, [c for c in \"ab\"])\n";
    let out = fixed(src);
    assert!(out.contains("var b = a.collect()"), "{out}");
    // A `Str` collects into what the context asks for, so it keeps only the help.
    assert!(out.contains("[c for c in \"ab\"]"), "{out}");
}

#[test]
fn loop_argmax_names_the_call() {
    // corpus noi03p4: `best` starts at `Home.first!`, so the call traps where it did.
    let src = "domain Home = 0..4\n\
let d: [Home]Int = read()\n\
var best = Home.first!\n\
for v in Home:\n    if d[v] > d[best]: best = v\n\
print(best)\n";
    assert!(
        fixed(src).contains("best = Home.max(by: v => d[v])!\nprint(best)"),
        "{}",
        fixed(src)
    );
    // acceptance run 4 cf2272A and std/geometry.cp: a range gets the exact call
    // as help, no fix, since an empty range kept `j` and the call traps.
    let range = "let p: 1..=9 = read()\n\
let a: [0..9]Int = read()\n\
var j = 0\n\
for i in 0..p if a[i] > a[j]:\n    j = i\n\
var s = 0\n\
for i in 1..p:\n    if a[i] < a[s]: s = i\n\
print(j, s)\n";
    let helps: Vec<String> = lint(range, &all_on())
        .into_iter()
        .filter(|d| d.code.0 == "L0547")
        .map(|d| d.help.unwrap_or_default())
        .collect();
    assert!(
        helps[0].contains("`var j = (0..p).max(by: i => a[i])!`"),
        "{helps:?}"
    );
    assert!(
        helps[1].contains("`var s = (0..p).min(by: i => a[i])!`"),
        "{helps:?}"
    );
    assert_eq!(fixed(range), range);
}

#[test]
fn float_equality_asks_the_checker_about_fields() {
    // std/geometry.cp: `Vec.x` is an `F64`, so every `.x` read as a float,
    // but `Pt.x` is an `Int`.
    let src = "struct Pt(x: Int, y: Int)\n\
struct Vec(x: F64, y: F64)\n\
let p = Pt(1, 2)\n\
let v = Vec(1.0, 2.0)\n\
print(yes(p.x == 0), yes(v.x == 0))\n";
    let found = lint(src, &all_on());
    let floats: Vec<_> = found
        .iter()
        .filter(|d| d.code == FLOAT_EQUALITY.code)
        .collect();
    assert_eq!(floats.len(), 1, "{found:#?}");
    assert!(floats[0].span.text(src).starts_with("v.x"), "{found:#?}");
}

#[test]
fn chmin_chmax_fix_keeps_the_line_break() {
    // corpus ioi08p3: a two-line `if` became `max_len[g] max= l` glued to the next line.
    let src = "var m = 0\nfor l in [3, 5]:\n    if l > m:\n        m = l\n\nprint(m)\n";
    assert_eq!(
        fixed(src),
        "var m = 0\nfor l in [3, 5]:\n    m max= l\n\nprint(m)\n"
    );
}

#[test]
fn chmin_chmax_needs_a_scalar() {
    // std/geometry.cp least_rotation: on lists `min=` is elementwise and `<`
    // lexicographic, so `best min= rot` returned a different rotation.
    let src = concat!(
        "let cyc = [3, 1, 2]\nvar best = [3, 1, 2]\nvar low = 9\n",
        "for s in 1..3:\n",
        "    let rot = [cyc[(s + i) % 3] for i in 0..3]\n",
        "    if rot < best: best = rot\n",
        "    best = min(best, [cyc[s], 0, 0])\n",
        "    if s < low: low = s\n",
        "print(best, low)\n",
    );
    let found: Vec<_> = lint(src, &all_on())
        .into_iter()
        .filter(|d| d.code == CHMIN_CHMAX_IF.code || d.code == COMPOUND_SELF_REF.code)
        .map(|d| d.span.text(src).trim().to_string())
        .collect();
    assert_eq!(found, ["if s < low: low = s"]);
}

/// A fold through a fn that widens the element (`Pos` into `?Pos`) is
/// suggested as a typed lambda, which checks; `fold(merge, None)` would not.
/// Rule: `lints/R9`.
#[test]
fn loop_reduction_keeps_a_widening_call_in_a_typed_lambda() {
    let src = "domain Pos = 0..4\nfn merge(a: ?Pos, b: ?Pos) -> ?Pos = if let x = a: x else: b\nvar root: ?Pos = None\nfor p in Pos:\n    root = merge(root, p)\nprint(root)\n";
    let found = format!("{:?}", crate::lint(src, &crate::Config::default()));
    assert!(
        found.contains("Pos.fold((root: ?Pos, p: Pos) => merge(root, p), None)"),
        "{found}"
    );
}

/// `desc:` is gone from `sort` (decisions 290, 366): a negated key, field or
/// not, becomes `desc(..)`. Rule: `idiomatic-code/R3`.
#[test]
fn sort_desc_by_negation_wraps_the_key_in_desc() {
    let src = "let ps = [(1, 2), (3, 4)]\nprint(ps.sorted(by: p => -p.1))\n";
    assert_eq!(
        only(src).help.as_deref(),
        Some("write `by: p => desc(p.1)`; `desc` reverses the order, and negating the key overflows on the smallest value")
    );
    assert_eq!(
        fixed(src),
        "let ps = [(1, 2), (3, 4)]\nprint(ps.sorted(by: p => desc(p.1)))\n"
    );
}

#[test]
fn sentinel_and_found_flag_read_a_written_type_from_the_checker() {
    let sentinel = |decl: &str| {
        let src = format!(
            "let n: 1..=100 = read()\nlet a: [0..n]Int = read()\n{decl}\nfor i in 0..n:\n    if a[i] == 0:\n        last = i\nprint(last)\n"
        );
        codes(&src).contains(&"L0536")
    };
    assert!(sentinel("var last = -1"));
    assert!(sentinel("var last: Int = -1"));
    assert!(!sentinel("var last: -1..=100 = -1"));
    assert!(!sentinel("var last: I128 = -1"));
    let flag = |decl: &str| {
        let src = format!(
            "let n: 1..=100 = read()\nlet a: [0..n]Int = read()\n{decl}\nvar last = 0\nfor i in 0..n:\n    if a[i] % 2 == 0:\n        last = a[i]\n        found = true\nif found == true:\n    print(last)\nelse:\n    print(-1)\n"
        );
        codes(&src).contains(&"L0596")
    };
    assert!(flag("var found = false"));
    assert!(flag("var found: Bool = false"));
    assert!(!flag("var found: ?Bool = false"));
}

#[test]
fn copy_comprehension_writes_collect_on_a_call_without_parens() {
    let src = "fn g() -> List<Int> = [3, 1]\nprint([x for x in g()])\n";
    assert_eq!(
        only(src).help.as_deref(),
        Some("write `g().collect()`: it copies any sequence into a new `List`")
    );
}

#[test]
fn manual_adjacency_list_reads_the_list_type_from_the_checker() {
    let fires = |decl: &str| {
        let src = format!(
            "let (n, m): (1..=200000, 0..=200000) = read()\n{decl}\nfor _ in 0..m:\n    let (u, v): (1..=n, 1..=n) = read()\n    adj[u].push(v)\nprint(adj[1].len)\n"
        );
        codes(&src).contains(&"L0537")
    };
    assert!(fires("var adj: List<List<Int>> = [[] for _ in 0..=n]"));
    assert!(fires("var adj: [0..=n]List<Int> = []"));
    assert!(!fires("var adj: List<Set<Int>> = [[] for _ in 0..=n]"));
}
