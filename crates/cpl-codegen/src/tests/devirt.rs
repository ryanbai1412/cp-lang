//! Fn values the C++ calls directly instead of through `std::function`.

use super::cpp;

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
        !out.contains("std::function") && !out.contains("std::get<"),
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
    let mut db = cpl_check::Database::new();
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
        "let k = read::<Int>()\n",
        "var b = Box(x => x + 1)\n",
        "if k > 3:\n    b = Box(x => x * k)\n",
        "print(b.op(k))\n",
    ));
    assert!(out.contains("std::function<ll(ll)> op;"), "{out}");
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
    let out = cpp("use std.algebra.{fold, opt}
let cat = (a: Int, b: Int) => a * 10 + b
let sub = (a: Int, b: Int) => a - b
print(fold(cat.opt(), [Some(1), None, Some(2)])!, fold(sub.opt(), [Some(5), Some(2)])!)
");
    assert!(
        !out.contains("std::function"),
        "each `opt()` clone captures one known closure, so its `op` calls it directly:\n{out}"
    );
}
