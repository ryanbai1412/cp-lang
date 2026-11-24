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

/// What `cpl explain` says of the fn-typed site `site` in `source`.
fn explained(source: &str, site: &str) -> crate::ClosureMode {
    let program = crate::tests::lower(source);
    let all = crate::explain_closures(&program);
    all.iter()
        .find(|e| e.site == site)
        .unwrap_or_else(|| panic!("no {site} in {all:?}"))
        .mode
        .clone()
}

#[test]
fn a_global_fn_set_once_is_called_as_its_closure() {
    let out = code(concat!(
        "let f = (x: Int) => x + 1\n",
        "let h = (x: Int) => x * 3\n",
        "fn g() -> Int:\n",
        "    f(2) + h(2)\n",
        "print(g())\n",
    ));
    assert!(out.contains("static auto cpl_fn0 = [](ll x)"), "{out}");
    assert!(out.contains("static auto cpl_fn1 = [](ll x)"), "{out}");
    assert!(out.contains("cpl_fn0(2), cpl_fn1(2)"), "{out}");
    assert!(!out.contains(" f;") && !out.contains(" h;"), "{out}");
    let src = "let f = (x: Int) => x + 1\nprint(f(2))\n";
    assert_eq!(
        explained(src, "top-level f"),
        crate::ClosureMode::StaticDirect { closure: 0 }
    );
}

#[test]
fn a_global_fn_set_twice_is_a_fn_pointer() {
    let src = concat!(
        "var f = (x: Int) => x + 1\n",
        "fn g() -> Int:\n",
        "    f(2)\n",
        "f = (x: Int) => x + 2\n",
        "print(g())\n",
    );
    let out = code(src);
    assert!(out.contains("std::add_pointer_t<ll(ll)> f;"), "{out}");
    assert!(out.contains("return f(2);"), "{out}");
    assert_eq!(explained(src, "top-level f"), crate::ClosureMode::FnPointer);
}

#[test]
fn a_fn_returning_one_closure_returns_its_lambda_unless_a_closure_or_itself_calls_it() {
    let out = code(concat!(
        "fn adder(k: Int) -> (Int) -> Int:\n",
        "    return (x) => x + k\n",
        "fn plain(k: Int) -> (Int) -> Int:\n",
        "    return (x) => x * k\n",
        "fn r(k: Int) -> (Int) -> Int:\n",
        "    if k > 0:\n",
        "        let _ = r(k - 1)\n",
        "    return (x) => x - k\n",
        "fn run(n: Int) -> Int:\n",
        "    let h = adder(n)\n",
        "    let p = plain(n)\n",
        "    let q = r(n)\n",
        "    let g = (y: Int) => adder(y)(1)\n",
        "    h(1) + p(2) + q(3) + g(4)\n",
        "print(run(3))\n",
    ));
    assert!(out.contains("inline auto plain("), "{out}");
    assert!(out.contains("auto p = plain(n);"), "{out}");
    let bound = "cpl::Bound<cpl::Env<ll>, ll(ll)>";
    assert!(out.contains(&format!("{bound} adder(")), "{out}");
    assert!(out.contains(&format!("{bound} h = adder(n);")), "{out}");
    assert!(out.contains(&format!("{bound} r(")), "{out}");
    assert!(out.contains(&format!("{bound} q = r(n);")), "{out}");
}

#[test]
fn a_reassigned_local_closure_is_not_auto() {
    let out = code(concat!(
        "fn run(n: Int) -> Int:\n",
        "    var f = (x: Int) => x + n\n",
        "    if n > 2:\n",
        "        f = (x: Int) => x * n\n",
        "    f(2)\n",
        "print(run(3))\n",
    ));
    assert!(
        out.contains("cpl::Bound<cpl::Env<ll>, ll(ll)> f = cpl::bind_env("),
        "{out}"
    );
}

#[test]
fn a_local_closure_a_nested_fn_reassigns_is_not_auto() {
    let nested = |body: &str| {
        code(&format!(
            "fn run(n: Int) -> Int:\n    fn go(m: Int) -> Int:\n{body}        f(2)\n    go(n) + go(1)\nprint(run(3))\n"
        ))
    };
    let once = nested("        let f = (x: Int) => x + m\n");
    assert!(once.contains("    auto f = [m_2 = m](ll x)"), "{once}");
    let twice = nested(concat!(
        "        var f = (x: Int) => x + m\n",
        "        if m > 2:\n",
        "            f = (x: Int) => x * m\n",
    ));
    assert!(
        twice.contains("cpl::Bound<cpl::Env<ll>, ll(ll)> f = cpl::bind_env("),
        "{twice}"
    );
}

#[test]
fn a_dropped_list_keeps_a_push_whose_index_can_trap() {
    let out = cpp("fn f(xs: List<Int>, i: Int) -> Int:
    var dead: List<Int> = []
    dead.push(xs[i])
    1
print(f([1, 2], read()))
");
    assert!(
        out.contains("cpl::at(xs, i)"),
        "`xs[i]` can trap, so the push onto the unread list stays:\n{out}"
    );
    let out = code(
        "domain D = 0..4
fn f(xs: [D]Int, i: D) -> Int:
    var dead: List<Int> = []
    dead.push(xs[i])
    1
print(f([D] d => 1, D.first!))
",
    );
    assert!(
        !out.contains("dead"),
        "`xs[i]` with `i: D` cannot trap, so the unread list goes:\n{out}"
    );
}

#[test]
fn a_struct_built_only_in_an_unwritten_fn_holds_no_fn() {
    let program = "struct S(f: (Int) -> Int)
fn mk(k: Int) -> S:
    S((x: Int) => x + k)
fn apply(s: S) -> Int:
    s.f(1)
let o: ?S = None
print(yes(o.is_none_or((s: S) => apply(s) > 0)))
";
    let out = cpp(program);
    assert!(
        out.contains("cpl::NoFn<ll> f;"),
        "only the unwritten `mk` builds an `S`:\n{out}"
    );
    let out = cpp(&format!("{program}print(apply(mk(read<Int>())))\n"));
    assert!(
        !out.contains("cpl::NoFn"),
        "`mk` runs, so `S.f` holds a closure:\n{out}"
    );
}

#[test]
fn a_global_closure_passed_as_an_argument_is_known_at_the_parameter() {
    let out = cpp("let g = (x: Int) => x + 1
fn apply(f: (Int) -> Int) -> Int:
    f(1)
print(apply(g))
");
    assert!(
        out.contains("apply([[maybe_unused]] bool f)"),
        "`apply` only ever gets the global `g`:\n{out}"
    );
}

#[test]
fn a_recursive_fn_called_with_two_closures_from_outside_its_cycle_stays_a_template() {
    let out = cpp("fn go(f: (Int) -> Int, n: Int) -> Int:
    if n == 0:
        return f(0)
    go(f, n - 1)
let k = read<Int>()
print(go((x: Int) => x + 1, k))
print(go((x: Int) => x * 2, k))
");
    assert!(
        out.contains("template <class F0>\nll go(F0&& f, ll n)"),
        "the recursion forwards `f`; outside callers may pass anything:\n{out}"
    );
}

#[test]
fn a_recursive_fn_local_to_a_local_fn_names_itself_through_cpl_rec() {
    let source = "fn outer(n: Int) -> Int:
    fn mid(k: Int) -> Int:
        fn rec(m: Int) -> Int:
            if m == 0: return 0
            rec(m - 1) + k
        rec(k)
    mid(n)
print(outer(read<Int>()))
";
    let out = cpp(source);
    assert!(
        out.contains("auto rec = cpl::rec([k_2 = k](auto& rec_2, ll m) mutable -> ll {"),
        "`cpl::rec` hands `rec` itself, so it is `auto`:\n{out}"
    );
    let program = crate::tests::lower(source);
    let all = crate::explain_closures(&program);
    let rec = all
        .iter()
        .find(|e| e.site.ends_with("local rec"))
        .unwrap_or_else(|| panic!("no local rec in {all:?}"));
    assert!(
        matches!(rec.mode, crate::ClosureMode::StaticLocalAuto),
        "{all:?}"
    );
}

#[test]
fn a_global_bound_to_a_method_of_a_global_keeps_its_receiver() {
    let src = "struct S:
    var k: Int
    fn get(self, x: Int) -> Int:
        x + k
let s = S(read<Int>())
let f = s.get
print(f(1))
";
    let out = code(src);
    assert_ne!(
        explained(src, "top-level f"),
        crate::ClosureMode::StaticDirect { closure: 0 },
        "{out}"
    );
}

#[test]
fn a_template_param_always_given_one_closure_may_go_unread() {
    let out = code(
        "fn apply(f: (Int) -> Int, x: Int) -> Int:
    let g = [f]
    g[0](x) + f(x)
let k = read<Int>()
print(apply((y: Int) => y + k, 2))
",
    );
    assert!(
        out.contains("ll apply([[maybe_unused]] F0&& f, ll x) {"),
        "{out}"
    );
}

#[test]
fn a_closure_in_an_unwritten_fn_needs_no_runtime() {
    let out = cpp("fn unused(n: Int) -> Int:
    let g = (q: HashMap<Int, Int>) => q.len
    g(HashMap<Int, Int>())
fn h(n: Int) -> Int:
    let f = (x: Int) => x + 1
    f(n)
print(h(read<Int>()))
");
    assert!(
        !out.contains("Salted hash"),
        "only `unused` holds a HashMap:\n{out}"
    );
}

#[test]
fn a_local_fn_and_the_closure_it_wraps_pass_each_other_only_when_nothing_else_names_them() {
    let head = "fn solve(n: Int) -> Int:
    let b = (x: Int) => if x == 0: n else: w(x - 1) + 1
";
    let pair = "auto w = cpl::memo(std::move(b";
    let lent = "auto b = w.body();";
    let erased = "std::function<ll(ll)> w;";
    for (rest, paired, named_after) in [
        ("    fn w(x: Int) -> Int = b(x)\n    w(3)\n", true, false),
        (
            "    fn w(x: Int) -> Int = b(x)\n    var t = 1\n    t = w(3)\n    t\n",
            true,
            false,
        ),
        ("    fn w(x: Int) -> Int = x + n\n    w(3)\n", false, false),
        (
            "    fn w(x: Int) -> Int = b(x)\n    w(3) + b(2)\n",
            true,
            true,
        ),
        (
            "    fn w(x: Int) -> Int = b(x)\n    let c = (y: Int) => b(y)\n    w(3) + c(1)\n",
            true,
            true,
        ),
    ] {
        let out = code(&format!("{head}{rest}print(solve(read<Int>()))\n"));
        assert_eq!(out.contains(pair), paired, "{rest}\n{out}");
        assert_eq!(out.contains(erased), !paired, "{rest}\n{out}");
        assert_eq!(out.contains(lent), named_after, "{rest}\n{out}");
    }
}

#[test]
fn a_cpp_fn_keeps_the_fn_param_its_body_was_written_against() {
    let out = code(
        "fn apply(f: (Int) -> Int, x: Int) -> Int = @cpp(\"return f(x);\")
print(apply((y: Int) => y + 1, read<Int>()))
",
    );
    assert!(
        out.contains("ll apply([[maybe_unused]] const std::add_pointer_t<ll(ll)>& f, [[maybe_unused]] ll x) {"),
        "{out}"
    );
}

#[test]
fn a_fresh_closure_from_outside_the_cycle_keeps_a_recursive_template() {
    let out = code(
        "fn rec(f: (Int) -> Int, n: Int) -> Int:
    if n == 0: return f(0)
    rec(f, n - 1) + 1
fn solve(n: Int) -> Int:
    let k = n * 2
    rec((x: Int) => x + k, n)
print(solve(read<Int>()))
",
    );
    assert!(out.contains("ll rec(F0&& f, ll n) {"), "{out}");
}

#[test]
fn a_fn_field_of_a_made_struct_is_its_makers_closure_even_when_the_field_varies() {
    let out = code(
        "struct Mon:
    op: (Int) -> Int
fn apply(f: (Int) -> Int, x: Int) -> Int:
    let g = [f]
    g[0](x) + f(x)
let k = read<Int>()
fn mk() -> Mon:
    return Mon((x: Int) => x + k)
fn solve(n: Int) -> Int:
    let a = mk()
    let ms = [a, Mon((x: Int) => x * k)]
    apply(a.op, n) + ms[n].op(n)
print(solve(read<Int>()))
",
    );
    assert!(
        out.contains("ll apply([[maybe_unused]] F0&& f, ll x) {"),
        "{out}"
    );
}

#[test]
fn a_dead_list_keeps_a_store_whose_index_can_trap() {
    let out = code(
        "domain D = 0..4
fn solve(n: Int, i: Int, j: D) -> Int:
    var d1: List<Int> = []
    d1[i] = 1
    var d3 = [D] d => 0
    d3[j] += n
    n + 1
print(solve(read<Int>(), read<Int>(), D.first!))
",
    );
    assert!(
        out.contains("cpl::at(d1, i) = 1;"),
        "`d1[i]` can trap:\n{out}"
    );
}

#[test]
fn a_top_level_closure_only_main_calls_is_auto_and_one_a_fn_reads_is_not() {
    let head = concat!(
        "struct S:\n    var k: Int\n    fn get(self, x: Int) -> Int:\n        x + k\n",
        "let s = S(read<Int>())\n",
        "let g = s.get\n",
    );
    let main_only = code(&format!("{head}print(g(2))\n"));
    assert!(main_only.contains("auto g = "), "{main_only}");
    let shared = code(&format!("{head}fn h(y: Int) -> Int = g(y)\nprint(h(2))\n"));
    assert!(!shared.contains("auto g = "), "{shared}");
}

#[test]
fn a_value_if_picks_closures_unless_one_is_a_local_the_branch_binds() {
    let free = code(
        "fn go(flip: Bool, xs: List<Int>) -> Int:
    let side = if flip: (t: Int) => xs[t] else: (t: Int) => t
    side(1)
print(go(read<Int>() > 0, [1, 2, 3]))
",
    );
    assert!(free.contains("auto side = cpl::pick("), "{free}");
    let bound = code(
        "fn go(flip: Bool, m: Int) -> Int:
    fn low(t: Int) -> Int = t + m
    let f = match flip:
        true =>
            let h = (t: Int) => t * m
            h
        false => low
    f(3)
print(go(read<Int>() > 0, 2))
",
    );
    assert!(
        !bound.contains("cpl::pick("),
        "`h` is set inside the join:\n{bound}"
    );
}

#[test]
fn a_struct_inside_a_key_keeps_every_field() {
    let fields = |uses: &str| {
        let out = cpp(&format!(
            "struct P:\n    a: Int\n    u: Int\nlet p = P(read<Int>(), 2)\n{uses}\n"
        ));
        let at = out.find("struct P {").expect("P");
        let end = at + out[at..].find("};").expect("P ends");
        out[at..end].contains("ll u")
    };
    assert!(!fields("print(p.a)"), "only `a` is read");
    for uses in [
        "let m: Map<P, Int> = [p: 1]\nprint(m.len, p.a)",
        "let m: Map<Int, Set<P>> = []\nprint(m.len, p.a)",
        "let m: Map<Int, Map<P, Int>> = []\nprint(m.len, p.a)",
        "let m: List<Set<P>> = []\nprint(m.len, p.a)",
        "let m: ?Set<P> = None\nprint(yes(m == None), p.a)",
        "let m: (Int, Set<P>) = (1, [])\nprint(m.0, p.a)",
        "let m: (x: Int, s: Set<P>) = (x: 1, s: [])\nprint(m.x, p.a)",
        "let f: ?(Set<P> -> Int) = None\nprint(yes(f == None), p.a)",
        "let f: ?(Int -> Set<P>) = None\nprint(yes(f == None), p.a)",
        "let m: HashSet<P> = []\nprint(m.len, p.a)",
        "let m: HashMap<P, Int> = []\nprint(m.len, p.a)",
        "let m: Set<(Int, P)> = []\nprint(m.len, p.a)",
        "let m: Set<(x: Int, q: P)> = []\nprint(m.len, p.a)",
        "let m: Set<?P> = []\nprint(m.len, p.a)",
        "let m: Set<List<P>> = []\nprint(m.len, p.a)",
        "let m: Set<Map<P, Int>> = []\nprint(m.len, p.a)",
        "let m: Set<Map<Int, P>> = []\nprint(m.len, p.a)",
    ] {
        assert!(
            fields(uses),
            "ordering or hashing a key reads all of P:\n{uses}"
        );
    }
}

#[test]
fn a_struct_a_fn_made_is_specialised_until_something_writes_it() {
    let go = |body: &str| {
        let out = cpp(&format!(
            "struct Mon:\n    var op: (Int, Int) -> Int\n\
             fn mk(k: Int) -> Mon = Mon((a: Int, b: Int) => a + b + k)\n\
             fn reset(mut m: Mon):\n    m = Mon((a: Int, b: Int) => a * b)\n\
             fn go(k: Int) -> Int:\n    var m = mk(k)\n{body}\n    m.op(1, 2)\n\
             print(go(read<Int>()))\n"
        ));
        let at = out.find("ll go(ll k) {").expect("go");
        out[at..at + out[at..].find("\n}").expect("go ends")].to_string()
    };
    for body in [
        "    let z = 0",
        "    let g = (x: Int) => x + k\n    print(g(1))",
    ] {
        let go = go(body);
        assert!(go.contains("Mon_k0_k0 m = mk(k);"), "{go}");
    }
    for body in [
        "    m = Mon((a: Int, b: Int) => a * b)",
        "    let g = () => m = Mon((a: Int, b: Int) => a * b)\n    g()",
        "    reset(mut m)",
        "    let g = () => m.op(0, 0)\n    print(g())",
    ] {
        let go = go(body);
        assert!(
            go.contains("Mon m = mk(k);"),
            "{body} may change `m.op`:\n{go}"
        );
    }
}

#[test]
fn closures_that_only_call_a_fn_with_their_own_params_are_one_fn() {
    let fns = |x: &str, y: &str| {
        let out = cpp(&format!(
            "fn merge(a: Int, b: Int) -> Int = a + b\nstruct Mon:\n    op: (Int, Int) -> Int\n\
             let k = read<Int>()\nlet x = Mon({x})\nlet y = Mon({y})\nprint(x.op(1, 2) + y.op(3, 4))\n"
        ));
        (out.contains("cpl_fn1"), out)
    };
    for y in ["merge", "(a: Int, b: Int) => merge(a, b)"] {
        let (two, out) = fns("merge", y);
        assert!(!two, "`{y}` is `merge`:\n{out}");
    }
    for y in [
        "(a: Int, b: Int) => merge(b, a)",
        "(a: Int, b: Int) => merge(a, k)",
    ] {
        let (two, out) = fns("merge", y);
        assert!(two, "`{y}` is not `merge`:\n{out}");
    }
}

#[test]
fn a_store_to_a_fn_place_joins_what_it_may_hold() {
    let go = |body: &str| {
        let out = cpp(&format!(
            "struct Mon:\n    var op: Int -> Int\n\
             fn setf(mut f: Int -> Int):\n    f = (x: Int) => x * 3\n\
             fn go(k: Int) -> Int:\n{body}\nprint(go(read<Int>()))\n"
        ));
        let at = out.find("ll go(").expect("go");
        out[at..at + out[at..].find("\n}").expect("go ends")].to_string()
    };
    let field = |store: &str| {
        go(&format!(
            "    var m = Mon((x: Int) => x + k)\n{store}    m.op(1)"
        ))
    };
    let local = |store: &str| go(&format!("    var g = (x: Int) => x + k\n{store}    g(1)"));
    let one = field("");
    assert!(one.contains("Mon_k0_k0 m = Mon_k0_k0{cpl_cl"), "{one}");
    let set = field("    m.op = (x: Int) => x * k\n");
    assert!(set.contains("Mon m = Mon{"), "{set}");
    let lent = field("    setf(mut m.op)\n");
    assert!(
        lent.contains("bind_env") && !lent.contains("{cpl_cl"),
        "{lent}"
    );
    let one = local("");
    assert!(one.contains("auto g = "), "{one}");
    for store in ["    g = (x: Int) => x * k\n", "    setf(mut g)\n"] {
        let many = local(store);
        assert!(
            many.contains("cpl::Bound<cpl::Env<ll>, ll(ll)> g = "),
            "{store}{many}"
        );
    }
}

#[test]
fn a_closure_reading_a_local_its_branch_writes_is_no_pick() {
    let picks = |write: &str| {
        cpp(&format!(
            "fn go(flip: Bool, m: Int) -> Int:\n    var z = m\n    let f = match flip:\n        true =>\n\
             {write}\n            (t: Int) => t * z\n        false => (t: Int) => t - m\n    f(3)\n\
             print(go(read<Int>() > 0, 2))\n"
        ))
        .contains("cpl::pick(")
    };
    assert!(picks("            let y = 1"));
    for write in [
        "            z = m + 1",
        "            for i in 0..m: z += i",
        "            while z < 10: z += 1",
        "            if m > 1: z = 1",
        "            if m > 1: print(1)\n            else: z = 1",
    ] {
        assert!(
            !picks(write),
            "the branch sets `z` before the closure reads it:\n{write}"
        );
    }
}

#[test]
fn an_unread_fill_is_dropped_only_when_it_is_small_and_runs_nothing() {
    let kept = |fill: &str| {
        let out = cpp(&format!("let k = read<Int>()\nlet xs = {fill}\nprint(k)\n"));
        out.lines().any(|l| l.trim_start().starts_with("xs = "))
    };
    for fill in [
        "[5].filled(0)",
        "[1048576].filled(0)",
        "[3, 4].filled(0)",
        "[5].filled([2].filled(0))",
    ] {
        assert!(!kept(fill), "{fill} is dropped");
    }
    for fill in [
        "[1048577].filled(0)",
        "[5].filled(read<Int>())",
        "[read<Int>()].filled(0)",
    ] {
        assert!(kept(fill), "{fill} is kept");
    }
}

#[test]
fn a_store_to_an_unread_field_keeps_only_its_value() {
    let out = cpp(
        "struct P:\n    a: Int\n    var u: Int\nvar p = P(read<Int>(), 2)\n\
         p.u = read<Int>()\nprint(p.a)\n",
    );
    assert!(out.contains("(void)(cpl::read_int());"), "{out}");
    assert!(
        !out.lines().any(|l| l.trim_start().starts_with("p.u")),
        "{out}"
    );
}

#[test]
fn a_fn_field_of_a_returned_struct_is_the_one_fn_every_return_puts_there() {
    let known = |loop_body: &str| {
        let out = cpp(&format!(
            "fn merge(a: Int, b: Int) -> Int = a + b\nfn other(a: Int, b: Int) -> Int = a * b\n\
             struct Mon:\n    id: Int\n    op: (Int, Int) -> Int\nfn base(k: Int) -> Mon = Mon(k, merge)\n\
             fn mk(flip: Bool) -> Mon:\n    for i in 0..read<Int>():\n{loop_body}\n    Mon(1, merge)\n\
             fn apply(f: (Int, Int) -> Int, x: Int) -> Int = f(x, x)\n\
             fn go(flip: Bool) -> Int:\n    let m = mk(flip)\n    let g = m.op\n    apply(g, 3)\n\
             print(go(read<Int>() > 0))\n"
        ));
        let at = out.find("ll apply(").expect("apply");
        out[at..at + out[at..].find("\n}").expect("apply ends")].contains("cpl_fn0(x, x)")
    };
    for body in [
        "        print(i)",
        "        if flip: return Mon(i, merge)",
        "        if flip: return base(i)",
    ] {
        assert!(known(body), "every return holds `merge`:\n{body}");
    }
    for body in [
        "        if flip: return Mon(i, other)",
        "        if flip: return Mon(i, merge)\n        if i > 2: return Mon(i, other)\n        if i > 3: return Mon(i, merge)",
    ] {
        assert!(!known(body), "a return holds `other`:\n{body}");
    }
}

#[test]
fn a_store_into_a_left_out_field_keeps_only_the_index_check() {
    let out = cpp(
        "struct S:\n    k: Int\n    var val: ()\nvar xs = [S(1, ()), S(2, ())]\n\
         xs[read<Int>()].val = ()\nvar s = S(3, ())\ns.val = ()\nprint(xs[0].k + s.k)\n",
    );
    assert!(out.contains("(void)cpl::at(xs, t0);"), "{out}");
    assert!(
        !out.lines()
            .any(|l| !l.trim_start().starts_with("//") && l.contains(".val")),
        "{out}"
    );
}
