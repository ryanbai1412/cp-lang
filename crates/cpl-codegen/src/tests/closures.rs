//! Fn values and closures lowered to templates, lambdas and fn pointers, not std::function.

use super::cpp;

#[test]
fn static_struct_and_param_ops_eliminate_std_function() {
    let src = concat!(
        "struct Box(op: (Int, Int) -> Int, base: Int)\n",
        "fn apply_box(b: Box, x: Int) -> Int = b.op(b.base, x)\n",
        "let b = Box((x, y) => x + y, 10)\n",
        "print(apply_box(b, 5))\n",
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let out = crate::emit(&program, "t.cp", None);
    assert!(!out.contains("std::function"), "{out}");
    assert!(out.contains("cpl_fn0("), "{out}");
    let explained = crate::explain_closures(&program);
    assert!(
        explained
            .iter()
            .any(|e| matches!(e.mode, crate::ClosureMode::StaticDirect { .. })),
        "{explained:?}"
    );
    assert!(
        explained
            .iter()
            .all(|e| !matches!(e.mode, crate::ClosureMode::DynamicStdFunction { .. })),
        "{explained:?}"
    );
}

#[test]
fn multi_closure_fn_param_lowers_to_template_functor() {
    let src = concat!(
        "fn twice(f: Int -> Int, x: Int) -> Int = f(f(x))\n",
        "let k = 3\n",
        "print(twice(x => x + 1, 2), twice(x => x + k, 2))\n",
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let out = crate::emit(&program, "t.cp", None);
    assert!(out.contains("template <class F0>"), "{out}");
    assert!(!out.contains("std::function"), "{out}");
    let explained = crate::explain_closures(&program);
    assert!(
        explained
            .iter()
            .any(|e| e.mode == crate::ClosureMode::StaticTemplate),
        "{explained:?}"
    );
}

#[test]
fn non_escaping_closure_borrows_immutable_non_scalar_captures() {
    let out = cpp("struct Box(xs: List<Int>):
    fn find_first(self, target: Int) -> Int:
        (0..self.xs.len).bsearch(i => self.xs[i] >= target)
let b = Box([1, 3, 5, 7])
print(b.find_first(4))
");
    assert!(
        out.contains("[&self_2 = self"),
        "expected non-escaping closure passed to bsearch_first to capture `self` by reference:\n{out}"
    );
}

#[test]
fn unread_fn_fields_are_dropped_instead_of_std_function() {
    let src = concat!(
        "struct Acc:\n",
        "    total: Int\n",
        "    var hook: ?(Int -> Int)\n",
        "struct Tagged:\n",
        "    pick: ?(Int -> Int)\n",
        "fn bump(d: Int) -> Int:\n",
        "    let f = (x: Int) => x + d\n",
        "    f(1)\n",
        "let k = read::<Int>()\n",
        "let a = Acc(1, x => x * k)\n",
        "var b = Acc(2, None)\n",
        "b.hook = x => x + k\n",
        "let t = Tagged(x => x - k)\n",
        "print(a.total + b.total, yes(t.pick == None), bump(k))\n",
    );
    let out = cpp(src);
    let acc = &out[out.find("struct Acc {").expect("Acc")..];
    assert!(!acc[..acc.find("};").unwrap()].contains("hook"), "{out}");
    assert!(
        out.contains("std::optional<std::function<ll(ll)>> pick;"),
        "{out}"
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    let mode = |site: &str| &explained.iter().find(|e| e.site == site).expect(site).mode;
    assert_eq!(
        mode("struct Acc.hook"),
        &crate::ClosureMode::Unread,
        "{explained:?}"
    );
    assert!(
        matches!(mode("struct Tagged.pick"), crate::ClosureMode::DynamicStdFunction { reason } if reason.contains("read")),
        "{explained:?}"
    );
}

#[test]
fn recursive_fn_threads_its_closure_param_as_a_template() {
    let src = concat!(
        "fn walk(n: Int, f: Int -> Int) -> Int:\n",
        "    if n == 0: return 0\n",
        "    f(n) + walk(n - 1, f)\n",
        "fn even(n: Int, f: Int -> Int) -> Int:\n",
        "    if n == 0: return f(0)\n",
        "    odd(n - 1, f)\n",
        "fn odd(n: Int, f: Int -> Int) -> Int:\n",
        "    if n == 0: return f(1)\n",
        "    even(n - 1, f)\n",
        "fn wrap(n: Int, f: Int -> Int) -> Int:\n",
        "    if n == 0: return f(0)\n",
        "    wrap(n - 1, x => f(x) + 1)\n",
        "let k: Int = read()\n",
        "print(walk(10, x => x * k), walk(5, x => x + k))\n",
        "print(even(7, x => x + k), odd(4, x => x * k))\n",
        "print(wrap(3, x => x + k))\n",
    );
    let out = cpp(src);
    assert!(out.contains("ll walk(ll n, F0&& f)"), "{out}");
    assert!(
        out.contains("ll even(ll n, F0&& f)") && out.contains("ll odd(ll n, F0&& f)"),
        "{out}"
    );
    assert!(
        out.contains("ll wrap(ll n, const std::function<ll(ll)>& f)"),
        "{out}"
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    let dynamic: Vec<_> = explained
        .iter()
        .filter(|e| matches!(e.mode, crate::ClosureMode::DynamicStdFunction { .. }))
        .collect();
    assert_eq!(dynamic.len(), 1, "{explained:?}");
    assert!(
        matches!(&dynamic[0].mode, crate::ClosureMode::DynamicStdFunction { reason } if reason.contains("different closure")),
        "{explained:?}"
    );
}

#[test]
fn recursive_local_closure_takes_itself_instead_of_std_function() {
    let src = concat!(
        "fn count(n: Int) -> Int:\n",
        "    var seen = 0\n",
        "    fn go(u: Int):\n",
        "        if u >= n: return\n",
        "        seen += 1\n",
        "        go(u * 2 + 1)\n",
        "        go(u * 2 + 2)\n",
        "    go(0)\n",
        "    seen\n",
        "print(count(10))\n",
    );
    let out = cpp(src);
    assert!(
        !out.contains("std::function") && !out.contains("<functional>"),
        "{out}"
    );
    assert!(out.contains("auto go = cpl::rec(["), "{out}");
    assert!(out.contains("](auto& go_2, ll u)"), "{out}");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    assert!(
        explained
            .iter()
            .all(|e| e.mode == crate::ClosureMode::StaticLocalAuto),
        "{explained:?}"
    );
}

#[test]
fn local_closure_lowers_to_auto_and_dynamic_field_falls_back_to_std_function() {
    let src = concat!(
        "struct Box(op: Int -> Int)\n",
        "fn run(k: Int) -> Int:\n",
        "    let loc = (x: Int) => x + k\n",
        "    loc(1)\n",
        "let k = 5\n",
        "var a = Box(x => x + 1)\n",
        "if k > 3:\n    a = Box(x => x + k)\n",
        "print(run(k), a.op(2))\n",
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let out = crate::emit(&program, "t.cp", None);
    assert!(out.contains("auto loc ="), "{out}");
    assert!(out.contains("std::function<ll(ll)> op;"), "{out}");
    let explained = crate::explain_closures(&program);
    assert!(
        explained
            .iter()
            .any(|e| e.mode == crate::ClosureMode::StaticLocalAuto),
        "{explained:?}"
    );
    assert!(
        explained
            .iter()
            .any(|e| matches!(e.mode, crate::ClosureMode::DynamicStdFunction { .. })),
        "{explained:?}"
    );
}

#[test]
#[ignore = "spec/conformance/std-lazy/flip-action-on-a-segtree.cp"]
fn higher_order_combinator_devirtualizes_to_zero_std_function() {
    let cpp = cpp(concat!(
        "use std.lazy.{Flip, flip_action}\n",
        "domain I = 0..4\n",
        "let m = Monoid<Int>(0, (a, b) => a + b)\n",
        "let act = flip_action<Int>(m)\n",
        "var st = SegTree<I, Flip<Int>>(act.over, [I] i => if i.ord % 2 == 0: Flip(1, 0) else: Flip(0, 1), act: act)\n",
        "st.apply(I(0)..I(3), true)\n",
        "print(st.all.now)\n",
    ));
    assert!(!cpp.contains("std::function"), "{cpp}");
    assert!(!cpp.contains("#include <functional>"), "{cpp}");
}

#[test]
fn a_yields_fn_no_loop_walks_has_no_callback_form() {
    let cpp = cpp(
        "fn squares(n: Int) -> yields Int:\n    for i in 0..n: yield i * i\n\
         let n: Int = read()\nprint(squares(n).collect().len)\n",
    );
    assert!(cpp.contains("squares(n)"), "{cpp}");
    assert!(!cpp.contains("cpl_each_squares"), "{cpp}");
}

#[test]
fn a_yields_loop_whose_body_calls_a_fn_value_runs_as_the_fn_yields() {
    let src = |closure: &str| {
        format!(
            "var seen = 0\nfn upto(n: Int) -> yields Int:\n    for i in 0..n:\n        seen += 1\n        yield i * seen\n\
             fn total(n: Int, w: Int -> Int) -> Int:\n    var t = 0\n    for x in upto(n): t += w(x)\n    t\n\
             print(total(4, {closure}))\n"
        )
    };
    let pure = cpp(&src("x => x * 2"));
    assert!(pure.contains("cpl_each_upto(n, "), "{pure}");
    assert!(!pure.contains("= upto(n)"), "{pure}");
    // The body would change what `upto` reads between its yields.
    let bumps = cpp(&src("x => { seen += 10; x }"));
    assert!(bumps.contains("= upto(n)"), "{bumps}");
}

#[test]
fn closure_mutating_a_captured_local_is_not_hoisted_static() {
    let cpp = cpp(concat!(
        "fn each(f: (Int) -> ()):\n",
        "    for i in 0..3: f(i)\n",
        "fn run() -> Int:\n",
        "    var xs = List<Int>()\n",
        "    each(x => xs.push(x))\n",
        "    var ys = List<Int>()\n",
        "    each(x => push_to(mut ys, x))\n",
        "    xs.len + ys.len\n",
        "fn push_to(mut l: List<Int>, x: Int):\n",
        "    l.push(x)\n",
        "print(run())\n",
    ));
    assert!(!cpp.contains("static auto cpl_fn"), "{cpp}");
}

#[test]
fn a_local_memo_body_is_an_auto_lambda_and_its_wrapper_is_explained() {
    let src = "let n: Int = read()\nfn solve() -> Int:\n    @memo\n    fn fib(i: 0..=n) -> Int = if i < 2: i else: fib(i - 1) + fib(i - 2)\n    fib(n)\nprint(solve())\n";
    let out = cpp(src);
    assert!(out.contains("auto __memo_body"), "{out}");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    let wrapper = explained
        .iter()
        .find(|e| e.site == "fn solve local fib")
        .expect("wrapper");
    assert!(
        matches!(&wrapper.mode, crate::ClosureMode::DynamicStdFunction { reason } if reason.contains("@memo")),
        "{explained:?}"
    );
}

#[test]
fn a_bound_method_value_is_its_lambda_not_a_std_function() {
    let src = concat!(
        "struct P:\n",
        "    w: Int\n",
        "    pub fn key(self, x: Int) -> Int = (x * w) % 7\n",
        "let p = P(read::<Int>())\n",
        "fn go(xs: List<Int>) -> List<Int> = xs.sorted(by: p.key)\n",
        "print(go([1, 2, 3, 4, 5]))\n",
    );
    let out = cpp(src);
    assert!(!out.contains("std::function"), "{out}");
}

#[test]
fn explain_names_closure_params_and_top_level_fn_vars() {
    let src = concat!(
        "fn go(k: Int):\n",
        "    let g = x => x + k\n",
        "    let h = (f: Int -> Int, x: Int) => f(x) * 2\n",
        "    print(h(g, 3), g(1))\n",
        "go(read::<Int>())\n",
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let reasons: Vec<String> = crate::explain_closures(&program)
        .into_iter()
        .filter_map(|e| match e.mode {
            crate::ClosureMode::DynamicStdFunction { reason } => Some(reason),
            _ => None,
        })
        .collect();
    assert!(
        reasons.iter().any(|r| r == "fn parameter of a closure"),
        "{reasons:?}"
    );
}

#[test]
fn a_closure_param_has_no_range_from_the_maker_slot_it_shares() {
    // Closure slot 0 is `a` and top-level slot 0 is `n`: `a`'s facts are
    // its own, so `a * b` is not taken to fit 32 bits from `n`'s range.
    let out = cpp(concat!(
        "let (n, m): (1..=1e5, 2..=1e9) = read()\n",
        "domain V = 1..=n\n",
        "let tree = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V.first!)\n",
        "let ways = tree.reroot(Monoid(1, (a, b) => a * b % m), (x, _, _) => (x + 1) % m, (u, _) => u)\n",
        "for v in V: print(ways[v])\n",
    ));
    assert!(!out.contains("uint32_t(a * b)"), "{out}");
}

#[test]
fn a_by_value_capture_keeps_the_facts_of_what_it_copies() {
    let out = cpp(concat!(
        "fn go(m: 1..=1000) -> Int:\n",
        "    let f = (x: 0..=1000) => (x * 7 + 1) % m\n",
        "    f(read()) + f(read())\n",
        "print(go(read()))\n",
    ));
    assert!(!out.contains("cpl::emod("), "{out}");
}

#[test]
fn the_emitter_asks_facts_only_through_its_own_key() {
    // A body's slots are keyed by `facts_home`: `self.fn_id` is the
    // enclosing fn's key inside a closure, and its slots are other variables.
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut bad = Vec::new();
    let mut stack = vec![src];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if !path.ends_with("tests") {
                    stack.push(path);
                }
            } else if path.extension().is_some_and(|e| e == "rs") && !path.ends_with("tests.rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                for (i, line) in text.lines().enumerate() {
                    if line.contains("self.facts.") && line.contains("self.fn_id") {
                        bad.push(format!("{}:{}", path.display(), i + 1));
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "use facts_home/facts_local: {bad:?}");
}
