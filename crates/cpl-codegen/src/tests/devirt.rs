//! Fn values the C++ calls directly instead of through `std::function`.

use super::{cpp, lower};

#[test]
fn a_fn_field_read_into_a_local_calls_its_only_closure() {
    let out = cpp("use std.tree.{Rooted}
use std.algebra.{Monoid}
domain V = 0..5
let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]
let t = Rooted(adj, root: 0)
print(t.reroot(Monoid(0, (a, b) => a + b), (c, _, _) => c + 1, (s, _) => s))
");
    assert!(
        !out.contains("f = mon.op"),
        "every `Monoid<Int>` holds the same `op`, so `f(..)` calls it directly:\n{out}"
    );
}

#[test]
fn a_fn_parameter_the_body_never_names_is_not_a_std_function() {
    let out = cpp("use std.heap.{Heap}
var h = Heap<Int>()
h.push(3)
h.push(1)
print(h.pop()!)
");
    assert!(
        !out.contains("std::function"),
        "`Heap.new` without `by:` never reads it, so it is a `bool` placeholder:\n{out}"
    );
}

#[test]
fn a_top_level_closure_only_called_is_not_a_std_function() {
    let out = cpp("let n = read<Int>()
let twice = (x: Int) => 2 * x
for i in 0..n:
    print(twice(i))
");
    assert!(
        !out.contains("std::function"),
        "`twice` is set once and only called, so its calls go to the closure:\n{out}"
    );
}

#[test]
fn a_tuple_of_closures_taken_apart_is_not_a_std_function() {
    let out = cpp("let n = read<Int>()
for t in 0..n:
    let (pre, suf) = ((i: Int) => i, (i: Int) => t + i)
    print(pre(t) + suf(t))
");
    assert!(
        !out.contains("std::function")
            && out.matches("std::get<").count()
                == out.matches("{ return std::get<K>(e); }").count(),
        "each part is bound from its own closure, never through a tuple:\n{out}"
    );
}

#[test]
fn an_optional_fn_param_every_call_fills_alike_is_called_directly() {
    let out = cpp(concat!(
        "fn top(xs: List<Int>, key: ?(Int -> Int) = None) -> Int:\n",
        "    if key != None:\n",
        "        return key!(xs[0])\n",
        "    xs[1]\n",
        "fn both(xs: List<Int>, cmp: ?((Int, Int) -> Bool)) -> Bool:\n",
        "    let less = (a: Int, b: Int) => if let c = cmp: c(a, b) else: a < b\n",
        "    less(xs[0], xs[1])\n",
        "let k = read::<Int>()\n",
        "print(top([4, 7]), yes(both([1, 2], (a, b) => a > b + k)))\n",
    ));
    assert!(!out.contains("std::function<"), "{out}");
    assert!(
        out.contains("inline ll top(const std::vector<ll>& xs) {\n  return cpl::at(xs, 1);"),
        "{out}"
    );
}

#[test]
fn a_fn_field_of_a_struct_never_built_is_no_std_function() {
    let out = cpp(concat!(
        "struct Op(f: (Int, Int) -> Int)\n",
        "struct Tree(xs: List<Int>, op: ?Op)\n",
        "fn grow(mut t: Tree, x: Int):\n",
        "    let o = t.op!\n",
        "    t.xs.push(o.f(x, x))\n",
        "var t = Tree(xs: [], op: None)\n",
        "let n = read::<Int>()\n",
        "if n > 100:\n",
        "    grow(mut t, n)\n",
        "print(t.xs.len, n)\n",
    ));
    assert!(!out.contains("std::function<"), "{out}");
    assert!(out.contains("cpl::NoFn<ll> f;"), "{out}");
}

#[test]
fn an_optional_fn_field_a_forwarded_none_fills_is_dropped() {
    let out = cpp(concat!(
        "var h = Heap.of([5, 3])\n",
        "h.push(read::<Int>())\n",
        "print(h.peek!)\n",
    ));
    assert!(!out.contains("std::function<"), "{out}");
}

#[test]
fn explain_lists_no_fn_field_of_a_struct_the_file_omits() {
    let src = concat!(
        "struct Unused(op: (Int, Int) -> Int)\n",
        "fn make(k: Int) -> Unused = Unused((x, y) => x + y + k)\n",
        "print(read::<Int>())\n",
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let out = crate::emit(&program, "t.cp", None);
    assert!(!out.contains("struct Unused"), "{out}");
    let explained = crate::explain_closures(&program);
    assert!(
        explained
            .iter()
            .all(|e| !e.site.starts_with("struct Unused")),
        "{explained:?}"
    );
}

#[test]
fn explain_lists_a_fn_that_returns_a_closure() {
    let src = concat!(
        "fn adder(n: Int) -> (Int -> Int):\n",
        "    (k: Int) => k * n + 1\n",
        "print(adder(read::<Int>())(2))\n",
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let out = crate::emit(&program, "t.cp", None);
    let explained = crate::explain_closures(&program);
    let listed = explained.iter().any(|e| {
        e.site == "fn adder result"
            && matches!(
                e.mode,
                crate::devirt::ClosureMode::DynamicStdFunction { .. }
            )
    });
    assert_eq!(
        listed,
        out.contains("std::function<"),
        "{explained:?}\n{out}"
    );
}

#[test]
fn a_struct_built_with_several_closures_splits_per_closure() {
    let out = cpp(concat!(
        "let k = read::<Int>()\n",
        "let add = Monoid<Int>.of(+)\n",
        "let big = Monoid<Int>.of(max)\n",
        "struct Box(op: Int -> Int)\n",
        "fn apply(b: Box, x: Int) -> Int = b.op(x)\n",
        "print(add.op(k, 2), big.op(k, 2), apply(Box(x => x + 1), k), apply(Box(x => x * k), k))\n",
    ));
    assert!(!out.contains("std::function<"), "{out}");
}

#[test]
fn a_variable_holding_two_closures_keeps_its_struct_whole() {
    let out = cpp(concat!(
        "struct Box(op: Int -> Int)\n",
        "fn go(k: Int):\n",
        "    var b = Box(x => x + 1)\n",
        "    if k > 3:\n        b = Box(x => x * k)\n",
        "    print(b.op(k))\n",
        "go(read::<Int>())\n",
    ));
    assert!(
        out.contains("cpl::Bound<cpl::Env<ll>, ll(ll)> op;"),
        "{out}"
    );
}

#[test]
fn each_clone_of_a_split_fn_gets_its_own_name() {
    let out = cpp(concat!(
        "struct Box(op: Int -> Int)\n",
        "fn walk(b: Box, x: Int, stop: Int -> Bool) -> Int:\n",
        "    var y = x\n",
        "    while not stop(y): y = b.op(y)\n",
        "    y\n",
        "let k = read::<Int>()\n",
        "let b = Box(x => x + 1)\n",
        "print(walk(b, k, y => y > 9), walk(b, k, y => y > k))\n",
    ));
    let heads: Vec<&str> = out
        .lines()
        .filter(|l| !l.starts_with(' ') && l.ends_with(") {") && l.contains("walk"))
        .collect();
    let unique: std::collections::BTreeSet<&&str> = heads.iter().collect();
    assert_eq!(heads.len(), unique.len(), "{out}");
}

#[test]
fn a_fn_field_behind_an_optional_struct_field_is_called_directly() {
    let out = cpp(concat!(
        "struct Act(f: Int -> Int)\n",
        "struct T(act: ?Act)\n",
        "fn go(t: T, x: Int) -> Int: t.act!.f(x)\n",
        "let k = read::<Int>()\n",
        "let a = T(Some(Act(x => x + 1)))\n",
        "let b = T(Some(Act(x => x * 2)))\n",
        "print(go(a, k), go(b, k))\n",
    ));
    assert!(!out.contains("std::function<"), "{out}");
    assert!(
        out.contains("static_cast<void>(cpl::have("),
        "the `!` still runs: {out}"
    );
}

#[test]
fn a_none_compared_with_a_split_optional_field_leaves_no_unsplit_struct() {
    let out = cpp(concat!(
        "struct Act(f: Int -> Int)\n",
        "struct T(act: ?Act)\n",
        "fn go(t: T, x: Int) -> Int: if t.act == None: x else: t.act!.f(x)\n",
        "let k = read::<Int>()\n",
        "let a = T(Some(Act(x => x + 1)))\n",
        "let b = T(Some(Act(x => x * 2)))\n",
        "print(go(a, k), go(b, k))\n",
    ));
    assert!(!out.contains("std::function<"), "{out}");
    assert!(
        !out.contains("struct Act {"),
        "only the split structs are defined: {out}"
    );
}

#[test]
fn a_fn_field_read_only_into_a_dropped_local_stores_no_std_function() {
    let out = cpp(concat!(
        "domain V = 0..3\n",
        "let adj: [V]List<V> = [[1, 2], [0], [0]]\n",
        "let t = Rooted(adj, root: 0)\n",
        "let win = t.up(+, lift: (w, c, p) => if w: 0 else: 1, finish: (losing, v) => losing > 0)\n",
        "print(sum(1 for v in V if win[v]))\n",
    ));
    assert!(!out.contains("std::function<"), "{out}");
}

#[test]
fn a_closure_capturing_a_known_fn_param_calls_it_directly() {
    let out = cpp("use std.algebra.{opt}
let cat = (a: Int, b: Int) => a * 10 + b
let sub = (a: Int, b: Int) => a - b
print([Some(1), None, Some(2)].fold(cat.opt())!, [Some(5), Some(2)].fold(sub.opt())!)
");
    assert!(
        !out.contains("std::function"),
        "each `opt()` clone captures one known closure, so its `op` calls it directly:\n{out}"
    );
}

#[test]
fn a_monoid_op_field_fed_only_captureless_closures_is_a_fn_pointer() {
    let out = cpp("use std.algebra.{Monoid}
struct Qs(m: Monoid<Int>)
let q = Qs(Monoid(0, (a, b) => min(a, b)))
let f = q.m.op
print(f(3, 4))
");
    assert!(
        !out.contains("std::function") && out.contains("std::add_pointer_t<ll(ll, ll)>"),
        "every 2-ary closure captures nothing, so `op` is a plain function pointer:\n{out}"
    );
}

#[test]
fn an_unbuilt_ring_still_converts_to_a_fn_pointer() {
    let out = cpp("type Z = Mod<7>
domain V = 0..2
print((Mat<Z>.new([V, V] (i, j) => Z(1)) ** 3).get(0, 0))
");
    assert!(
        !out.contains("std::function") && out.contains("cpl::NoFn"),
        "a `Mat` without a ring never calls its `add`/`mul`:\n{out}"
    );
}

#[test]
fn a_capturing_closure_of_the_same_arity_keeps_std_function() {
    let out = cpp("fn go(k: Int):
    let fs = [(x: Int) => x + k, (x: Int) => x * 2]
    for f in fs:
        print(f(3))
go(read<Int>())
");
    assert!(
        !out.contains("std::add_pointer_t"),
        "a capturing 1-ary closure can't be a function pointer:\n{out}"
    );
}

#[test]
fn a_local_read_only_as_a_direct_callee_is_not_declared() {
    let out = cpp(include_str!(
        "../../../cpl-cli/tests/programs/operator_value_default.cp"
    ));
    assert!(
        !out.contains("& f = cpl::have(self.act)"),
        "`f.apply(..)` calls its closure directly, so `f` is never read; only `act!` is checked:\n{out}"
    );
    assert!(
        out.contains("cpl::have(self.act);"),
        "the `act!` check stays:\n{out}"
    );
}

#[test]
fn a_capturing_closure_of_another_signature_leaves_a_monoid_op_a_fn_pointer() {
    let out = cpp("use std.algebra.{Monoid}
struct Qs(m: Monoid<Int>)
fn tag(s: Str):
    let f = (t: Str, n: Int) => s + t + \"{n}\"
    print(f(\"a\", 1))
let q = Qs(Monoid(0, (a, b) => min(a, b)))
let f = q.m.op
tag(read<Str>())
print(f(3, 4))
");
    assert!(
        out.contains("std::add_pointer_t<ll(ll, ll)>"),
        "only closures of `op`'s own signature decide its representation:\n{out}"
    );
}

#[test]
fn a_capturing_closure_no_running_code_makes_leaves_a_monoid_op_a_fn_pointer() {
    let out = cpp("use std.algebra.{Monoid}
struct Qs(m: Monoid<Int>)
fn unused(k: Int) -> Int:
    let g = (a: Int, b: Int) => a + b + k
    g(1, 2)
let q = Qs(Monoid(0, (a, b) => min(a, b)))
let f = q.m.op
print(f(3, read<Int>()))
");
    assert!(
        out.contains("std::add_pointer_t<ll(ll, ll)>") && !out.contains("std::function<"),
        "a closure in a fn nothing calls decides no representation:\n{out}"
    );
}

#[test]
fn an_optional_fn_field_only_none_fills_is_no_std_function() {
    let out = cpp("domain D = 0..4
let init = [D] _ => 0
var st = SegTree<D, Int, Int>(min, act: add, a: init)
fn first_le(mut st: SegTree<D, Int, Int>, t: Int) -> Int = (0..4).first_true(m => st.get(D(m)) <= t) ?? 4
st.apply(0..2, 1)
print(first_le(mut st, 1))
");
    assert!(
        !out.contains("std::function"),
        "`source` is None in every tree the program builds:\n{out}"
    );
}

#[test]
fn an_optional_fn_field_of_tuple_t_only_none_fills_is_no_std_function() {
    let out = cpp("domain D = 0..4
type P = (a: Int, b: Int)
let onto = Action<P, Int>(
    id: 0,
    compose: (newer, older) => newer + older,
    apply: (f, p, _) => (a: p.a + f, b: p.b),
    over: Monoid((a: 0, b: 0), (x, _) => x),
)
let init = [D] _ => (a: 0, b: 0)
var st = SegTree<D, P, Int>(act: onto, a: init)
fn first_le(mut st: SegTree<D, P, Int>, t: Int) -> Int = (0..4).first_true(m => st.get(D(m)).a <= t) ?? 4
st.apply(0..2, 1)
print(first_le(mut st, 1))
");
    assert!(
        !out.contains("std::function"),
        "`source` holds tuples but is None in every tree the program builds:\n{out}"
    );
}

#[test]
fn a_dropped_field_at_a_product_position_drops_its_local_and_stores() {
    let out = cpp(concat!(
        "use std.graph.{fixed_cost_paths}\n",
        "domain R = 0..5\n",
        "domain T = 0..3\n",
        "let p = fixed_cost_paths([(R(0), Subset<T>.empty)], [1, 2], cost: m => m, ",
        "step: (m, s) => if Int(s.0) + m < 5: Some((R(Int(s.0) + m), s.1)) else: None)\n",
        "print(p[(R(4), Subset<T>.empty)])\n",
    ));
    assert!(
        !out.contains(" back(") && !out.contains("back["),
        "nothing reads `Paths.back`, so neither its array nor a store into it is written:\n{out}"
    );
}

#[test]
fn a_field_only_an_unwritten_method_reads_is_dropped() {
    let out = cpp(concat!(
        "domain V = [5]\n",
        "let g: [V]List<(to: V, w: Int)> = [[(to: V(1), w: 2)], [(to: V(2), w: 3)], [], [], []]\n",
        "print(dijkstra(g, from: [(V(0), 0)])[V(2)]!)\n",
    ));
    assert!(
        !out.contains("order.push_back") && !out.contains("> order"),
        "only `Paths.iter` reads `order`, and the file never writes it, so neither the field nor its pushes are kept:\n{out}"
    );
}

#[test]
fn a_closure_named_to_keep_argument_order_is_no_std_function() {
    let out = cpp("use std.search.{parallel_bsearch}
let n = read<Int>()
let off = n - 1
let w = [n for _ in 0..n]
let need = [n for _ in 0..n]
var cnt = 0
print(yes(parallel_bsearch(need.len, off..off + n, reset: () => cnt = 0, apply: i => cnt += w[i - off], ok: q => cnt >= need[q]) == []))
");
    assert!(
        !out.contains("std::function"),
        "a closure argument named before the call keeps its lambda type:\n{out}"
    );
}

#[test]
fn an_op_value_field_is_left_out_of_declaration_literal_and_tie() {
    let out = cpp("use std.window.{Window}
var w = Window(+)
for x in [3, 5, 2]: w.push(x)
print(w)
");
    assert!(
        !out.contains("void op"),
        "an op value holds nothing:\n{out}"
    );
    assert!(
        !out.contains("{(void)0,"),
        "the literal drops the field too:\n{out}"
    );
}

#[test]
fn a_closure_calling_a_known_field_of_a_returned_struct_is_no_std_function() {
    let out = cpp("use std.lazy.{Flip}
let m = Flip.op(+)
let x = m.op(Flip(3, 7), m.id)
print(x.now, x.flipped, m.id.now)
");
    assert!(
        !out.contains("std::function"),
        "`Flip.op(+)` captures a Monoid whose `op` is one known fn, so its own `op` is static:\n{out}"
    );
}

#[test]
fn a_fn_returning_one_closure_returns_its_lambda_type() {
    let out = cpp("fn adder(n: Int) -> (Int -> Int):
    (k: Int) => k * n + 1
fn twice(n: Int) -> Int:
    let f = adder(n)
    f(f(1))
print(adder(3)(2), twice(2))
");
    assert!(
        out.contains("auto adder(") && !out.contains("std::function"),
        "`adder` makes one closure at its one return, so it returns `auto`:\n{out}"
    );
}

#[test]
fn a_closure_capturing_an_unwrapped_known_param_captures_nothing() {
    let out = cpp("use std.heap.Heap
var h: Heap<Int> = Heap(by: x => -x)
h.push(1)
h.push(5)
print(h.pop()!)
");
    assert!(
        !out.contains("std::function"),
        "`by` is one known closure, so the comparator made from it captures nothing:\n{out}"
    );
}

#[test]
fn a_closure_capturing_only_zero_size_values_captures_nothing() {
    let out = cpp("let m = Monoid<(Int, Int, Int)>.of(each(+, max, min))
print(m.op((1, 2, 3), (4, 0, 9)))
");
    assert!(
        !out.contains("std::function"),
        "`each(+, max, min)` holds only op values, so its op captures nothing:\n{out}"
    );
}

#[test]
fn closures_stored_in_a_list_of_records_are_fn_pointers_or_bound() {
    let out = cpp(concat!(
        "fn mk(k: Int) -> List<(n: Int, f: Int -> Int, g: (Int, Int) -> Int)> = [(n: 1, f: x => x + k, g: (a, b) => a + b), (n: 2, f: x => x * k, g: (a, b) => a * b)]\n",
        "for r in mk(read::<Int>()): print(r.n, r.f(3), r.g(3, 4))\n",
    ));
    assert!(!out.contains("std::function<"), "{out}");
    assert!(out.contains("cpl::Bound<cpl::Env<ll>, ll(ll)>"), "{out}");
    assert!(out.contains("std::add_pointer_t<ll(ll, ll)>"), "{out}");
}

#[test]
fn an_unread_flat_key_is_not_computed() {
    let out = cpp(
        "domain R = 0..3\ndomain C = 2..=4\nvar g: [R, C]Int = 0\nfor p in g.keys:\n    g[p] = p.0.label\nprint(g[(R(1), C(2))])\n",
    );
    for line in out.lines().map(str::trim) {
        assert!(!line.starts_with("(int)((ll)"), "{line}");
    }
    assert!(!out.contains("C_len + (ll)"), "{out}");
}

#[test]
fn each_sink_reports_every_let_loop_binding_store_and_return() {
    use crate::uses::split::{each_sink, Sink};
    let program = lower(concat!(
        "fn f(n: Int) -> Int:\n",
        "    var s = 0\n",
        "    for i in 0..n: s += i\n",
        "    return s\n",
        "print(f(read::<Int>()))\n",
    ));
    let f = program.fns.iter().find(|f| f.name == "f").expect("f");
    let mut kinds = Vec::new();
    each_sink(&f.body, &mut |s| {
        kinds.push(match s {
            Sink::Let(..) => "let",
            Sink::For(_) => "for",
            Sink::Store(..) => "store",
            Sink::Return(_) => "return",
        })
    });
    assert_eq!(kinds, ["let", "for", "store", "return"]);
}

#[test]
fn each_value_reports_an_expression_after_the_ones_inside_it() {
    let program = lower("let k = read::<Int>()\nprint(k * (k + 1), [k, 2 * k].len)\n");
    let mut order: Vec<*const cpl_ir::Expr> = Vec::new();
    crate::uses::split::each_value(&program.main, &mut |e| order.push(e));
    let at = |e: &cpl_ir::Expr| order.iter().position(|&p| std::ptr::eq(p, e));
    let mut checked = 0;
    crate::uses::split::each_value(&program.main, &mut |e| {
        crate::walk::children(e, &mut |child| {
            assert!(at(child) < at(e), "a child after its parent");
            checked += 1;
        });
    });
    assert!(checked > 4, "{checked} children");
}

#[test]
fn a_fn_param_never_named_stays_elided_when_a_closure_makes_the_call() {
    let out = cpp("fn run(n: Int, g: Int -> Int) -> Int = n * 2
let k = read<Int>()
let go = (m: Int) => run(m, x => x + k)
print(go(k))
");
    assert!(
        !out.contains("std::function"),
        "`run` never names `g`, and the one call (inside `go`) passes a closure:\n{out}"
    );
}
