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
        out.contains("std::optional<cpl::Bound<cpl::Env<ll>, ll(ll)>> pick;"),
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
        mode("struct Tagged.pick") == &crate::ClosureMode::Bound,
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
fn local_closure_lowers_to_auto_and_a_field_of_like_closures_is_a_bound_fn() {
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
    assert!(
        out.contains("cpl::Bound<cpl::Env<ll>, ll(ll)> op;"),
        "{out}"
    );
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
            .any(|e| e.mode == crate::ClosureMode::Bound),
        "{explained:?}"
    );
}

#[test]
fn higher_order_combinator_devirtualizes_to_zero_std_function() {
    let cpp = cpp(concat!(
        "use std.lazy.{flip_action}\n",
        "domain I = 0..4\n",
        "let act = flip_action<Int>((a, b) => a + b)\n",
        "var st = SegTree<I, Flip<Int>, Bool>(act.over, [I] i => if i.ord % 2 == 0: Flip(1, 0) else: Flip(0, 1), act: act)\n",
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
fn a_getter_generator_take_stops_after_k_like_a_call() {
    let cpp = cpp(
        "struct W:\n    xs: List<Int>\n    @get pub fn evens(self) -> yields Int:\n        for x in xs:\n            if x % 2 == 0: yield x\n\
         let n: Int = read()\nlet w = W([i for i in 0..n])\nprint(w.evens.take(8).count())\n",
    );
    assert!(cpp.contains("cpl_each_W_evens"), "{cpp}");
    assert!(
        !cpp.contains("std::vector<ll> W_evens("),
        "the whole getter is built: {cpp}"
    );
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
fn a_fed_loop_calling_a_fn_value_sees_only_that_fn_types_effects() {
    let cpp = cpp(concat!(
        "var g = [1, 2, 3]\n",
        "fn gen() -> yields Int:\n    for x in g: yield x\n",
        "fn bump(x: Int) -> Int:\n    if x < 3: g.push(x + 10)\n    x\n",
        "fn show(s: Bool) -> Bool: s\n",
        "fn run(f: Int -> Int, h: Bool -> Bool) -> Int:\n",
        "    var t = 0\n    for x in gen(): t = t * 100 + f(x)\n    if h(true): t else: 0\n",
        "print(run(bump, show), g.len)\n",
    ));
    // `f` grows `g` while `gen` walks it, so the walk is not fed.
    assert!(!cpp.contains("cpl_each_gen("), "{cpp}");
}

#[test]
fn a_nested_fn_writing_the_callers_var_still_streams_a_generator() {
    let cpp = cpp(concat!(
        "fn solve(n: Int) -> Int:\n",
        "    let h = [i % 7 for i in 0..n]\n",
        "    var found = 0\n",
        "    fn crossed(i: Int, j: Int):\n        if (h[i] + h[j]) % 3 == 0: found += 1\n",
        "    let groups = (0..n).group(by: x => h[x])\n",
        "    for (_, group) in groups:\n        for (i, j) in group.pairs(): crossed(i, j)\n",
        "    found\n",
        "let n: 1..=1000 = read()\nprint(solve(n))\n",
    ));
    assert!(cpp.contains("cpl_each_m1_pairs"), "{cpp}");
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
fn a_local_memo_fn_and_its_body_pass_each_other_instead_of_a_std_function() {
    let src = "let n: Int = read()\nfn solve() -> Int:\n    @memo\n    fn fib(i: 0..=n) -> Int = if i < 2: i else: fib(i - 1) + fib(i - 2)\n    fib(n)\nprint(solve())\n";
    let out = cpp(src);
    assert!(out.contains("auto __memo_body"), "{out}");
    assert!(
        out.contains("auto fib = cpl::memo(std::move(__memo_body"),
        "{out}"
    );
    assert!(!out.contains("std::function"), "{out}");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    let wrapper = explained
        .iter()
        .find(|e| e.site == "fn solve local fib")
        .expect("wrapper");
    assert!(
        matches!(&wrapper.mode, crate::ClosureMode::StaticLocalAuto),
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
        "fn go(k: Int, m: Int):\n",
        "    let g = x => x + k\n",
        "    let e = (x: Int) => x + k * m\n",
        "    let h = (f: Int -> Int, x: Int) => f(x) * 2\n",
        "    print(h(g, 3), h(e, 3), g(1))\n",
        "go(read::<Int>(), read::<Int>())\n",
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

/// A C++ local a template declares around interpolated code can hide the
/// user's local of that name (`size_t at` in a print block hid `at`), so
/// every such name is `cpl_*`, which `names::ident` never gives a user's.
#[test]
fn every_local_a_template_declares_is_cpl_prefixed() {
    const TYPES: [&str; 7] = ["auto", "auto&", "auto&&", "size_t", "ll", "int", "bool"];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut stack = vec![root];
    let mut bad = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("src dir") {
            let path = entry.expect("entry").path();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if path.is_dir() {
                if name != "tests" && name != "runtime" {
                    stack.push(path);
                }
                continue;
            }
            if !name.ends_with(".rs") || name.ends_with("tests.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("source");
            let lines = text.lines().take_while(|l| l.trim() != "#[cfg(test)]");
            for (n, line) in lines.enumerate() {
                for lit in line.split('"').skip(1).step_by(2) {
                    let interpolates = lit.replace("{{", "").contains('{');
                    let spaced = lit.replace("\\n", " ");
                    let words: Vec<&str> = spaced.split_whitespace().collect();
                    for (k, w) in words.windows(3).enumerate() {
                        let next = words.get(k + 3).copied().unwrap_or("");
                        let decl = TYPES.contains(&w[1]) && !matches!(w[0], "inline" | "static");
                        let id: String = w[2]
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                            .collect();
                        let rest = &w[2][id.len()..];
                        let ends = rest.starts_with([';', '=', ':'])
                            || rest.is_empty() && next.starts_with(['=', ':']);
                        let temp =
                            id.starts_with('t') && id[1..].chars().all(|c| c.is_ascii_digit());
                        if decl
                            && interpolates
                            && ends
                            && !id.is_empty()
                            && !id.starts_with("cpl_")
                            && !temp
                        {
                            bad.push(format!("{}:{}: {id}", path.display(), n + 1));
                        }
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "name these cpl_*:\n{}", bad.join("\n"));
}

/// A macro the emitted file defines rewrites any user name spelled the
/// same, so `names::ident` gives no user name the spelling of one.
#[test]
fn no_user_name_is_spelled_like_an_emitted_macro() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut stack = vec![root];
    let mut macros = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("src dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            let text = text.split("#[cfg(test)]").next().unwrap_or_default();
            for part in text.split("#define ").skip(1) {
                let name: String = part
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                macros.push(name);
            }
        }
    }
    assert!(macros.iter().any(|m| m == "LOCAL"), "found {macros:?}");
    let clash: Vec<&String> = macros
        .iter()
        .filter(|m| crate::names::ident(m) == **m)
        .collect();
    assert!(clash.is_empty(), "add to names::RESERVED: {clash:?}");
}

#[test]
fn a_yields_fn_whose_own_closures_write_its_locals_feeds_a_loop() {
    let cpp = cpp(concat!(
        "fn halves(n: Int) -> yields Int:\n",
        "    var todo: List<Int> = []\n",
        "    fn add(x: Int):\n",
        "        todo.push(x)\n",
        "    add(n)\n",
        "    while let x = todo.pop():\n",
        "        yield x\n",
        "        if x > 1: add(x / 2)\n",
        "let n: 1..=1000000 = read()\n",
        "for x in halves(n):\n",
        "    if x < n: break\n",
        "    print(x)\n",
    ));
    let solve = &cpp[cpp.find("void solve()").unwrap()..];
    assert!(solve.contains("cpl_each_halves(n, "), "{solve}");
}

#[test]
fn a_value_if_of_different_closures_joins_at_the_erased_fn() {
    let out = cpp(concat!(
        "let m = read<Int>()\n",
        "fn go(flip: Bool) -> Int:\n",
        "    fn low(t: Int) -> Int = t + m\n",
        "    let side = if flip: (t: Int) => m - low(t) else: low\n",
        "    side(3)\n",
        "print(go(true), go(false))\n",
    ));
    assert!(
        out.contains("side = flip ? std::function<ll(ll)>("),
        "{out}"
    );
    assert!(out.contains(" : std::function<ll(ll)>(low)"), "{out}");
}

#[test]
fn a_yields_fn_that_walks_another_feeds_its_callback_on() {
    let cpp = cpp(concat!(
        "fn inner2(n: Int) -> yields Int:\n    for i in 0..n: yield i\n",
        "fn outer2(n: Int) -> yields Int:\n    for x in inner2(n): yield x\n",
        "let n: Int = read()\nvar t = 0\nfor x in outer2(n): t += x\nprint(t)\n",
    ));
    let outer = cpp.rsplit("int cpl_each_outer2(").next().unwrap();
    let outer = &outer[..outer.find("\n}\n").unwrap()];
    assert!(outer.contains("cpl_each_inner2(n_2, "), "{outer}");
    assert!(!outer.contains("std::vector"), "{outer}");
}

#[test]
fn a_block_local_generator_feeds_its_loop_like_a_top_level_one() {
    let cpp = cpp(concat!(
        "fn first(n: Int) -> Int:\n    var seen = 0\n",
        "    fn nat() -> yields Int:\n        var i = 0\n",
        "        while true:\n            seen += 1\n            yield i\n            i += 1\n",
        "    for x in nat():\n        if x == n: break\n    seen\n",
        "let n: Int = read()\nprint(first(n))\n",
    ));
    let caller = cpp.rsplit("ll first(").next().unwrap();
    let caller = &caller[..caller.find("\n}\n").unwrap()];
    assert!(caller.contains("cpl_each_nat_local"), "{caller}");
    assert!(!caller.contains("std::vector"), "{caller}");
}

#[test]
fn a_struct_fn_field_stored_to_later_is_not_one_closures_type() {
    let out = cpp(concat!(
        "struct Op1:\n",
        "    var f: (Int) -> Int\n",
        "let k: Int = read()\n",
        "var b = Op1(x => x + k)\n",
        "b.f = x => x * k\n",
        "print(b.f(2))\n",
    ));
    assert!(
        !out.contains("cpl_cl0 f;"),
        "b.f holds two closures:\n{out}"
    );
}

#[test]
fn a_struct_field_built_from_one_capturing_closure_is_that_closures_type() {
    let src = concat!(
        "struct Plus(z: Int):\n",
        "    fn monoid(self) -> Monoid<Int> = Monoid(z, (a, b) => self.op(a, b))\n",
        "impl Op<Int> for Plus:\n",
        "    fn op(self, a: Int, b: Int) -> Int = a + b\n",
        "let n: 1..=8 = read()\n",
        "let m = Plus(n).monoid()\n",
        "print(m.op(m.id, 2))\n",
    );
    let out = cpp(src);
    assert!(!out.contains("std::function"), "{out}");
    assert!(out.contains("cpl_cl0 op;"), "{out}");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    assert!(
        explained
            .iter()
            .any(|e| matches!(e.mode, crate::ClosureMode::StaticFunctor { closure: 0 })),
        "{explained:?}"
    );
}

#[test]
fn a_struct_field_two_closures_with_like_captures_reach_is_a_bound_fn() {
    let src = concat!(
        "struct Op2(f: (Int, Int) -> Int)\n",
        "fn run(k: Int):\n",
        "    let ops = [Op2((a, b) => a + b + k), Op2((a, b) => a * b * k)]\n",
        "    for o in ops: print(o.f(2, 3))\n",
        "let k: 1..=8 = read()\n",
        "run(k)\n",
    );
    let out = cpp(src);
    assert!(
        out.contains("cpl::Bound<cpl::Env<ll>, ll(ll, ll)> f;"),
        "{out}"
    );
    assert!(!out.contains("std::function<"), "{out}");
    assert!(!out.contains("cpl_cl"), "{out}");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    assert!(
        explained
            .iter()
            .any(|e| e.site == "struct Op2.f" && e.mode == crate::ClosureMode::Bound),
        "{explained:?}"
    );
}

#[test]
fn a_fed_body_that_never_stops_says_so_in_its_callback_type() {
    let cpp = cpp(concat!(
        "fn upto3(n: Int) -> yields Int:\n    for i in 0..n: yield i\n",
        "let n: Int = read()\nvar t = 0\n",
        "for x in upto3(n):\n    if x % 2 == 0: continue\n    t += x\n",
        "for x in upto3(n):\n    if x > 5: break\n    t += x\n",
        "print(t)\n",
    ));
    let calls: Vec<&str> = cpp
        .lines()
        .filter(|l| l.contains("cpl_each_upto3(n, "))
        .collect();
    assert_eq!(calls.len(), 2, "{cpp}");
    assert!(
        calls[0].contains("-> std::integral_constant<int, 0> {"),
        "{cpp}"
    );
    assert!(calls[1].contains("-> int {"), "{cpp}");
}

/// The lines of `cpl_each_{name}`'s form that call a `cpl_each_` form.
fn pusher_calls(cpp: &str, name: &str) -> Vec<String> {
    let head = format!("int cpl_each_{name}(");
    let form: Vec<&str> = cpp
        .lines()
        .skip_while(|l| !(l.starts_with("template") && l.contains(&head) && l.ends_with('{')))
        .take_while(|l| *l != "}")
        .collect();
    assert!(!form.is_empty(), "no cpl_each_{name} in\n{cpp}");
    form.iter()
        .filter(|l| l.contains("= cpl_each_"))
        .map(|l| l.to_string())
        .collect()
}

#[test]
fn a_recursive_generator_hands_itself_one_callback_type() {
    let cpp = cpp(concat!(
        "fn walk(n: Int) -> yields Int:\n    yield n\n",
        "    if n > 0:\n        for x in walk(n - 1): yield x\n",
        "fn f() -> Int:\n    for x in walk(10):\n        if x == 7: return x\n    0\n",
        "print(f())\n",
    ));
    // A fresh lambda per level would make g++ instantiate the form forever.
    let calls = pusher_calls(&cpp, "walk");
    assert_eq!(calls.len(), 1, "{cpp}");
    assert!(
        calls[0].contains("cpl_each_walk(cpl::isub(n, 1), cpl::fn_ref([&]"),
        "{cpp}"
    );
    assert_eq!(cpp.matches("cpl_each_walk(").count(), 3, "{cpp}");
}

#[test]
fn mutually_recursive_generators_hand_on_one_callback_type() {
    let cpp = cpp(concat!(
        "fn odd(n: Int) -> yields Int:\n    if n > 0:\n        yield n\n",
        "        for x in even(n - 1): yield x\n",
        "fn even(n: Int) -> yields Int:\n    if n > 0:\n        yield -n\n",
        "        for x in odd(n - 1): yield x\n",
        "var s = 0\nfor x in odd(9): s += x\nprint(s)\n",
    ));
    for (name, other) in [("odd", "even"), ("even", "odd")] {
        let calls = pusher_calls(&cpp, name);
        assert_eq!(calls.len(), 1, "{cpp}");
        assert!(calls[0].contains(&format!("cpl_each_{other}(")), "{cpp}");
        assert!(calls[0].contains("cpl::fn_ref([&]"), "{cpp}");
    }
}

#[test]
fn a_generator_over_a_recursive_one_passes_its_own_lambda() {
    let cpp = cpp(concat!(
        "fn walk(n: Int) -> yields Int:\n    yield n\n",
        "    if n > 0:\n        for x in walk(n - 1): yield x\n",
        "fn twice() -> yields Int:\n    for x in walk(3): yield 2 * x\n",
        "var s = 0\nfor x in twice(): s += x\nprint(s)\n",
    ));
    let calls = pusher_calls(&cpp, "twice");
    assert_eq!(calls.len(), 1, "{cpp}");
    assert!(calls[0].contains("cpl::fn_ref([&]"), "{cpp}");
}

#[test]
fn explain_lists_a_fn_inside_a_record_that_stays_std_function() {
    let src = concat!(
        "fn mk(k: Int, m: Int) -> List<(n: Int, f: Int -> Int)> = [(n: 1, f: x => x + k), (n: 2, f: x => x * k * m)]\n",
        "let (k, m): (Int, Int) = read()\n",
        "for r in mk(k, m): print(r.n, r.f(3))\n",
    );
    let out = cpp(src);
    assert!(out.contains("std::function<ll(ll)>"), "{out}");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    assert!(
        explained
            .iter()
            .any(|e| e.site.contains("inside a tuple, record or container")
                && matches!(e.mode, crate::ClosureMode::DynamicStdFunction { .. })),
        "{explained:?}"
    );
}

#[test]
fn a_name_for_a_template_fn_parameter_is_that_functor_not_a_std_function() {
    let out = cpp(concat!(
        "fn each(xs: [2]Int, f: Int -> Int) -> List<Int> = [y for y in xs.map(f)]\n",
        "let k: Int = read()\n",
        "print(each([1, 2], v => v + k), each([3, 4], v => v * k))\n",
    ));
    assert!(!out.contains("std::function"), "{out}");
}

#[test]
fn a_top_level_fn_let_only_main_uses_is_its_closures_own_type() {
    let src = concat!(
        "fn adder(k: Int) -> (Int -> Int) = x => x + k\n",
        "let n: Int = read()\n",
        "let f = adder(n)\n",
        "let g = (x: Int) => x * n\n",
        "print(f(3), g(2))\n",
    );
    let out = cpp(src);
    assert!(!out.contains("std::function"), "{out}");
    assert!(out.contains("auto f = adder("), "{out}");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let explained = crate::explain_closures(&program);
    let f = explained
        .iter()
        .find(|e| e.site == "top-level f")
        .expect("f");
    assert!(
        matches!(f.mode, crate::ClosureMode::StaticLocalAuto),
        "{explained:?}"
    );
}

#[test]
fn local_fns_that_call_each_other_pass_each_other_instead_of_a_std_function() {
    let out = cpp(concat!(
        "fn walk(n: Int, k: Int) -> Int:\n",
        "    fn down(i: Int) -> Int = if i <= 0: 0 else: k + up(i - 1)\n",
        "    fn up(i: Int) -> Int = if i <= 0: 0 else: 1 + down(i - 1)\n",
        "    down(n) + up(n)\n",
        "let (n, k): (Int, Int) = read()\n",
        "print(walk(n, k))\n",
    ));
    assert!(!out.contains("std::function"), "{out}");
    assert!(out.contains("cpl::memo("), "{out}");
}

#[test]
fn a_closure_passed_in_an_optional_the_callee_only_calls_borrows_its_list() {
    let source = concat!(
        "fn total(n: Int, weight: ?(Int -> Int) = None) -> Int:\n",
        "    var t = 0\n",
        "    for i in 0..n:\n",
        "        if let w = weight: t += w(i)\n",
        "        else: t += 1\n",
        "    t\n",
        "fn go(k: Int) -> Int:\n",
        "    let xs = [k, 2, 3]\n",
        "    var s = 0\n",
        "    for weighted in [false, true]:\n",
        "        s += total(3, weight: if weighted: Some((i: Int) => xs[i]) else: None)\n",
        "    s\n",
        "print(go(read<Int>()))\n",
    );
    let out = cpp(source);
    assert!(
        out.contains("[&xs_2 = xs]"),
        "the closure lives only for the call to `total`, so it borrows `xs`:\n{out}"
    );
    let explained = crate::explain_copies(&super::lower(source));
    assert!(
        explained.is_empty(),
        "no copy is left to explain: {explained:?}"
    );
}
