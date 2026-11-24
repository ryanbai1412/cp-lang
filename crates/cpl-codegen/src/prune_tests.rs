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
fn a_return_a_known_branch_always_takes_drops_the_rest_of_the_fn() {
    // cf2229i: `convolve(over: Semiring(max, +))` returns from its first branch, so
    // the NTT path after it and the fns only that path calls are not emitted.
    let cpp = cpp(concat!(
        "fn same<A, B>(a: A, b: B) -> Bool = a == b\n",
        "fn fast(a: Int, b: Int) -> Int = a * b + 1\n",
        "fn slow(a: Int, b: Int) -> Int = a * b + 2\n",
        "fn combine<F>(a: Int, b: Int, on: F = +) -> Int:\n",
        "    if not same(on, +):\n",
        "        return slow(a, b)\n",
        "    fast(a, b)\n",
        "let n: 1..=10 = read()\n",
        "print(combine(n, n, on: max))\n",
    ));
    let code: String = cpp.lines().filter(|l| !l.starts_with("//")).collect();
    assert!(code.contains("slow"), "{cpp}");
    assert!(!code.contains("fast"), "{cpp}");
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
    assert!(solve.contains("put_int(0, '\\n')"), "{cpp}");
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
    let mut db = cpl_check::Session::new();
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

fn unused_runtime(file: &str) -> Vec<&str> {
    crate::runtime::shake::unused(file)
        .into_iter()
        .map(|at| &file[at])
        .collect()
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
        unused_runtime(file),
        ["// `first` of a view: its first element\ntemplate <class T> T first(T v) { return v; }\n"]
    );
}

#[test]
fn a_reached_fn_template_keeps_its_whole_body() {
    let file = concat!(
        "namespace cpl {\n",
        "template <class S, class F> void sort_by(S& seq, F&& key) {\n",
        "    using K = std::decay_t<decltype(key(*seq.begin()))>;\n",
        "    if constexpr (std::is_void_v<K>) {\n",
        "        return;\n",
        "    } else {\n",
        "    std::vector<K> keys;\n",
        "    std::stable_sort(seq.begin(), seq.end());\n",
        "    }\n",
        "}\n",
        "}  // namespace cpl\n",
        "int main() { std::vector<int> v; cpl::sort_by(v, [](int x) { return x; }); }\n",
    );
    assert_eq!(unused_runtime(file), Vec::<&str>::new());
}

#[test]
fn a_runtime_struct_and_the_fns_only_it_reaches_go_together() {
    let file = concat!(
        "namespace cpl {\n",
        "inline int helper(int x) { return x; }\n",
        "template <class T> struct Box {\n",
        "    T v;\n",
        "    int get() const { return helper(1); }\n",
        "};\n",
        "template <> struct Box<bool> {\n",
        "    bool v;\n",
        "};\n",
        "inline int used(int x) {\n",
        "    return x + 1;\n",
        "}\n",
        "}  // namespace cpl\n",
        "int main() { return cpl::used(0); }\n",
    );
    let unused = unused_runtime(file);
    assert_eq!(unused.len(), 3, "{unused:?}");
    assert!(unused[0].starts_with("inline int helper"));
    assert!(unused[1].starts_with("template <class T> struct Box"));
    assert!(unused[2].starts_with("template <> struct Box<bool>"));
}

#[test]
fn a_reached_struct_keeps_what_its_members_call_and_a_global_stays() {
    let file = concat!(
        "namespace cpl {\n",
        "inline int helper(int x) { return x; }\n",
        "struct Box {\n",
        "    int get() const { return helper(1); }\n",
        "};\n",
        "inline bool operator<(Box, Box) { return false; }\n",
        "inline int counter = 0;\n",
        "#ifdef LOCAL\n",
        "inline void trace() {}\n",
        "#endif\n",
        "}  // namespace cpl\n",
        "int main() { cpl::Box b; return b.get(); }\n",
    );
    assert_eq!(unused_runtime(file), ["inline void trace() {}\n"]);
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
fn a_comparison_with_none_of_a_field_only_none_fills_keeps_only_the_branch_taken() {
    let cpp = cpp(concat!(
        "struct Box:\n",
        "    f: ?Int\n",
        "fn get(b: Box) -> Int:\n",
        "    if b.f != None: return b.f! * 7\n",
        "    if None == b.f: return 3\n",
        "    5\n",
        "let n: 1..=10 = read()\n",
        "print(get(Box(None)) + n)\n",
    ));
    let start = cpp.find(" get(").expect("a get fn");
    let get = &cpp[start..start + cpp[start..].find("\n}").expect("its end")];
    assert!(!get.contains("has_value()"), "{get}");
    assert!(
        get.contains("return 3") && !get.contains("return 5"),
        "{get}"
    );
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

#[test]
fn a_skipped_assert_still_runs_a_condition_that_stores() {
    let source = "use std.graph.{floyd_warshall}
domain V = 0..3
var hop: [V, V]?Int = None
hop[0, 1] = 1
hop[1, 0] = 1
hop[1, 2] = 1
hop[2, 1] = 1
assert floyd_warshall(mut hop)
assert hop[0, 2] == Some(2)
print(hop[0, 2] ?? -1)
";
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let target = crate::Target {
        checks: crate::Checks::None,
        ..crate::Target::default()
    };
    let out = crate::emit_for(&program, "t", None, &target);
    let body = &out[out.find("void solve()").unwrap_or(0)..];
    assert!(body.contains("floyd_warshall"), "{out}");
    assert!(!body.contains("assertion failed"), "{out}");
}

const POOL: &str = concat!(
    "struct Pool:\n",
    "    xs: List<Int> = []\n",
    "    n: Int\n",
    "    fn grow(mut self):\n",
    "        xs.push(n)\n",
    "    fn spread(self) -> Int = xs.len * 7 + 3\n",
    "    fn get(mut self) -> Int:\n",
    "        if xs.len > 0:\n",
    "            self.grow()\n",
    "            return self.spread()\n",
    "        n\n",
    "let n: 1..=10 = read()\n",
    "var s = Pool(n: n)\n",
);

#[test]
fn a_branch_on_the_len_of_a_list_field_nothing_fills_is_left_out() {
    // A store only the left-out branch makes does not keep it: `grow` is reached only from there.
    let cpp = cpp(&format!("{POOL}print(s.get())\n"));
    let code: String = cpp.lines().filter(|l| !l.starts_with("//")).collect();
    assert!(code.contains("Pool_get"), "{cpp}");
    assert!(
        !code.contains("Pool_grow") && !code.contains("Pool_spread"),
        "{cpp}"
    );
}

#[test]
fn a_list_field_code_that_runs_fills_keeps_its_branches() {
    let cpp = cpp(&format!("{POOL}if n > 5: s.grow()\nprint(s.get())\n"));
    let code: String = cpp.lines().filter(|l| !l.starts_with("//")).collect();
    assert!(code.contains("Pool_spread"), "{cpp}");
}

#[test]
fn a_lazy_segtree_over_a_domain_emits_only_the_dense_tree() {
    // atcoder_dp_w: the `over:` layout's growing tree went out with the dense one.
    let cpp = cpp(concat!(
        "let n: 1..=2e5 = read()\n",
        "domain P = 0..=n\n",
        "var t = SegTree<P, Int, Int>(max, fill: 0, act: add)\n",
        "t.apply(.., 1)\n",
        "print(t.fold(..))\n",
    ));
    for growing in [
        "_grow(",
        "_open_node(",
        "_blank_at(",
        "_kid(",
        "_dense_width(",
    ] {
        assert!(!cpp.contains(growing), "{growing} in {cpp}");
    }
    assert!(cpp.len() < 26 * 1024, "{} bytes", cpp.len());
}

#[test]
fn a_none_passed_on_through_a_parameter_still_drops_the_branch_on_it() {
    let cpp = cpp(concat!(
        "fn slow(a: Int) -> Int = a * a + 2\n",
        "fn inner(a: Int, b: ?Int) -> Int:\n",
        "    if let v = b: return slow(v + a)\n",
        "    a + 1\n",
        "fn outer(a: Int, b: ?Int) -> Int = inner(a, b)\n",
        "let n: 1..=10 = read()\n",
        "print(outer(n, None))\n",
    ));
    let code: String = cpp.lines().filter(|l| !l.starts_with("//")).collect();
    assert!(!code.contains("slow"), "{cpp}");
}

#[test]
fn a_let_of_a_param_nobody_passes_goes_with_the_branches_testing_it() {
    let get = |body: &str| {
        let cpp = cpp(&format!(
            "fn get(x: Int, by: ?Int = None) -> Int:\n{body}\n    x + 1\nprint(get(read<Int>()))\n"
        ));
        let start = cpp.find("ll get(").expect("a get fn");
        cpp[start..start + cpp[start..].find("\n}").expect("its end")].to_string()
    };
    for body in [
        "    let f = by\n    if f != None: return x * f!",
        "    if x > 3:\n        let f = by\n        if f != None: return x * f!",
        "    if x > 3: print(1)\n    else:\n        let f = by\n        if f != None: return x * f!",
        "    for i in 0..x:\n        let f = by\n        if f != None: return i * f!",
        "    while x > 3:\n        let f = by\n        if f != None: return x * f!",
    ] {
        let get = get(body);
        assert!(!get.contains(" f = by;"), "{body}\n{get}");
    }
    let kept = get(
        "    let f = by\n    let g = () => f\n    if f != None: return x * f!\n    if g() != None: return 2",
    );
    assert!(kept.contains(" f = by;"), "a closure reads `f`:\n{kept}");
}

#[test]
fn a_skipped_assert_that_stores_still_runs_inside_a_branch_or_loop() {
    for wrap in [
        "if n > 0:\n    ",
        "if n > 5: print(0)\nelse:\n    ",
        "while n > 5:\n    ",
        "for i in 0..n:\n    ",
    ] {
        let source = format!(
            "use std.graph.{{floyd_warshall}}\ndomain V = 0..3\nvar hop: [V, V]?Int = None\n\
             hop[0, 1] = 1\nhop[1, 2] = 1\nlet n: 1..=10 = read()\n\
             {wrap}assert floyd_warshall(mut hop)\nprint(hop[0, 2] ?? -1)\n"
        );
        let mut db = cpl_check::Session::new();
        db.set_file("t.cp", source.as_str());
        let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
        let target = crate::Target {
            checks: crate::Checks::None,
            ..crate::Target::default()
        };
        let out = crate::emit_for(&program, "t", None, &target);
        let body = &out[out.find("void solve()").unwrap_or(0)..];
        assert!(body.contains("floyd_warshall"), "{source}\n{out}");
    }
}

fn solve_body(source: &str, checks: crate::Checks) -> String {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let target = crate::Target {
        checks,
        ..crate::Target::default()
    };
    let out = crate::emit_for(&program, "t", None, &target);
    out[out.find("void solve()").unwrap_or(0)..].to_string()
}

#[test]
fn an_assert_storing_through_the_fns_it_calls_still_runs_or_is_checked() {
    let source = "use std.graph.{floyd_warshall}\ndomain V = 0..3\nvar hop: [V, V]?Int = None\n\
                  hop[0, 1] = 1\nhop[1, 2] = 1\n\
                  fn close(mut d: [V, V]?Int) -> Bool = floyd_warshall(mut d)\n\
                  fn wrap(mut d: [V, V]?Int) -> Bool = close(mut d)\n\
                  assert wrap(mut hop)\nprint(hop[0, 2] ?? -1)\n";
    let unchecked = solve_body(source, crate::Checks::None);
    assert!(unchecked.contains("wrap("), "{unchecked}");
    assert!(!unchecked.contains("assertion failed"), "{unchecked}");
    let checked = solve_body(source, crate::Checks::default());
    assert!(
        checked.contains("wrap(") && checked.contains("assertion failed"),
        "{checked}"
    );
}

fn tests_some(source: &str) -> bool {
    let cpp = cpp(source);
    let start = cpp.find("ll get(").expect("a get fn");
    let get = &cpp[start..start + cpp[start..].find("\n}").expect("its end")];
    get.contains("has_value()") || get.contains("* 7") || get.contains(", 7)")
}

#[test]
fn a_field_read_from_input_at_any_depth_may_hold_something() {
    let source = |read: &str, built: &str| {
        format!(
            "struct B:\n    f: ?Int\n    g: ?Int\nstruct C:\n    b: B\n\
             fn get(b: B) -> Int:\n    if b.f != None: return b.f! * 7\n    5\n\
             {read}let q = B(None, Some(read<Int>()))\nlet w = {built}\nprint(get(w) + get(q))\n"
        )
    };
    assert!(!tests_some(&source("", "B(None, None)")));
    for read in [
        "B",
        "List<B>",
        "(Int, B)",
        "List<List<B>>",
        "(x: Int, b: B)",
        "C",
    ] {
        let read = format!("let r: {read} = read()\n");
        assert!(tests_some(&source(&read, "B(None, None)")), "{read}");
    }
    for built in ["B(q.f, None)", "B(None, q.f)"] {
        assert!(
            !tests_some(&source("", built)),
            "{built} fills `f` with None"
        );
    }
    assert!(tests_some(&source("", "B(q.g, None)")), "`g` is Some");
}

#[test]
fn an_optional_known_none_is_one_nothing_can_set() {
    let get = |body: &str, params: &str| {
        tests_some(&format!(
            "fn get(x: Int{params}) -> Int:\n{body}\n    x + 1\nprint(get(read<Int>()))\n"
        ))
    };
    let by = ", by: ?Int = None";
    assert!(!get(
        "    let f: ?Int = None\n    if f != None: return x * f!",
        ""
    ));
    assert!(!get("    if by != None: return x * by!", by));
    assert!(get(
        "    if x > 5: by = Some(2)\n    if by != None: return x * by!",
        ", var by: ?Int = None"
    ));
    assert!(get(
        "    var g = by\n    let h = () => g = Some(3)\n    h()\n    if g != None: return x * g!",
        by
    ));
}

#[test]
fn a_construction_counts_only_where_code_can_reach_it() {
    let some = |body: &str| {
        tests_some(&format!(
            "struct B:\n    f: ?Int\nfn get(b: B) -> Int:\n    if b.f != None: return b.f! * 7\n    5\n\
             fn mk(x: Int) -> B:\n{body}\n    B(Some(x))\nprint(get(mk(read<Int>())))\n"
        ))
    };
    for body in [
        "    return B(None)",
        "    if x > 0: return B(None)\n    else: return B(None)",
        "    if 1 < 2: return B(None)",
        "    if 1 > 2: print(0)\n    else: return B(None)",
    ] {
        assert!(!some(body), "`B(Some(x))` is never reached:\n{body}");
    }
    for body in [
        "    if x > 0: return B(None)",
        "    for i in 0..x:\n        if i > 2: return B(Some(i))",
        "    while x > 3:\n        return B(Some(x))",
        "    let h = () => B(Some(x))\n    return h()",
        "    let h = () => B(Some(x))\n    return B(None)",
    ] {
        assert!(some(body), "a `Some` is reached:\n{body}");
    }
}
