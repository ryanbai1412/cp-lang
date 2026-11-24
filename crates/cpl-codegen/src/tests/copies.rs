//! Lists, maps and views passed by reference or moved, never copied.

use super::{body, cpp};

#[test]
fn a_reversed_list_pays_only_for_the_view_parts_it_uses() {
    let read = "let n: Int = read()\nlet xs: List<Int> = [read() for _ in 0..n]\n";
    let size = |body: &str| cpp(&format!("{read}{body}")).len();
    let forward = size("var s = 0\nfor x in xs: s = s * 31 + x\nprint(s)\n");
    let backward = size("var s = 0\nfor x in xs.rev: s = s * 31 + x\nprint(s)\n");
    assert!(backward <= forward + 256, "{forward} -> {backward}");
    let plain = size("let ys = xs\nvar s = 0\nfor x in ys: s = s * 31 + x\nprint(s, ys[0])\n");
    let view = cpp(&format!(
        "{read}let ys = xs.rev\nvar s = 0\nfor x in ys: s = s * 31 + x\nprint(s, ys.nth(0))\n"
    ));
    assert!(view.len() <= plain + 3200, "{plain} -> {}", view.len());
    assert!(
        !view.contains("list_concat") && !view.contains("list_slice"),
        "{view}"
    );
}

#[test]
fn a_local_list_that_stays_in_its_fn_is_one_buffer() {
    let fixture = include_str!("../../../cpl-cli/tests/programs/reused_local_lists.cp");
    let cpp = cpp(fixture);
    assert!(
        cpp.contains("static std::vector<ll> xs;\n  xs.clear();"),
        "{cpp}"
    );
    assert!(!cpp.contains("static std::vector<ll> seen;"), "{cpp}");
    assert!(!cpp.contains("static std::vector<ll> out;"), "{cpp}");
}

#[test]
fn a_reused_row_of_lists_is_cleared_before_it_is_filled_but_a_flat_row_is_not() {
    let cpp = cpp(concat!(
        "let n: 1..=100 = read()\n",
        "domain Row = 0..n\n",
        "var total = 0\n",
        "for r in 0..n:\n",
        "    var rows: [Row]List<Int> = []\n",
        "    rows[Row(r)].push(r)\n",
        "    var seen: [Row]Int = 0\n",
        "    seen[Row(r)] = r\n",
        "    for row in rows: total += row.len\n",
        "    for x in seen: total += x\n",
        "print(total)\n",
    ));
    assert!(cpp.contains("    seen.assign(Row_len, 0);"), "{cpp}");
    assert!(
        cpp.contains("static std::vector<std::vector<ll>> rows;\n    rows.clear(); rows.assign(Row_len, std::vector<ll>());"),
        "{cpp}"
    );
}

#[test]
fn a_list_a_comprehension_built_moves_into_the_variable() {
    let cpp = cpp("let n: Int = read()
var zs: List<Int> = []
for r in 0..3:
    zs = [i * r for i in 0..n]
    let ys = [z + 1 for z in zs]
    zs = ys
print(zs)
");
    assert!(cpp.contains("zs = std::move(t0);"), "{cpp}");
    assert!(
        cpp.contains("zs = std::move(ys);"),
        "ys is not read again: {cpp}"
    );
}

#[test]
fn a_let_of_a_list_nothing_changes_is_a_reference_not_a_copy() {
    let cpp = cpp(concat!(
        "let n: Int = read()\n",
        "var rows: List<List<Int>> = [[i * j for j in 0..=i] for i in 0..n]\n",
        "fn last_of(r: Int) -> Int:\n",
        "    let row = rows[r]\n",
        "    row[row.len - 1]\n",
        "print(last_of(2))\n",
        "let (_, vals) = compress([30, 10, 20, 10])\n",
        "print(vals.index_of(20))\n",
    ));
    assert!(
        cpp.contains("const std::vector<ll>& row = cpl::at(rows, r);"),
        "{cpp}"
    );
    // `vals.index_of(x)` searches the field where it is.
    assert!(!cpp.contains("= self.vals;"), "{cpp}");
}

#[test]
fn an_if_let_payload_nothing_changes_is_a_reference_not_a_copy() {
    let cpp = cpp(concat!(
        "fn total(n: Int, end: ?List<Int>) -> Int:\n",
        "    var s = 0\n",
        "    for i in 0..n:\n",
        "        if let e = end: s += e[i % e.len]\n",
        "    s\n",
        "var best: ?List<Int> = Some([1])\n",
        "fn swap() -> Int:\n",
        "    if let kept = best:\n",
        "        best = Some([2, 3])\n",
        "        return kept.len\n",
        "    0\n",
        "print(total(5, Some([1, 2, 3])), swap())\n",
    ));
    assert!(
        cpp.contains("const cpl::Lent<std::vector<ll>>& t0 = end;"),
        "{cpp}"
    );
    assert!(cpp.contains("const std::vector<ll>& e = *t0;"), "{cpp}");
    // `swap` changes `best`: the optional is copied before the test.
    assert!(
        cpp.contains("std::optional<std::vector<ll>> t0 = best;"),
        "{cpp}"
    );
}

#[test]
fn a_let_at_the_top_level_is_a_reference_while_nothing_after_it_changes_the_list() {
    let cpp = cpp(concat!(
        "let n: Int = read()\n",
        "var rows: [0..n]List<Int> = []\n",
        "fn bump(i: Int):\n    rows[i].push(100)\n",
        "fn size(i: Int) -> Int: rows[i].len\n",
        "var total = 0\n",
        "for i in 0..n:\n    let row = rows[i]\n    total += row.len + size(i)\n",
        "fn inside(i: Int) -> Int:\n",
        "    if i > 0:\n        let a = rows[i]\n        return a.len + size(i)\n",
        "    bump(i)\n    rows[i].len\n",
        "print(total, inside(0))\n",
    ));
    // `size` reads `rows` and changes nothing.
    assert!(
        cpp.contains("const std::vector<ll>& row = cpl::at(rows, "),
        "{cpp}"
    );
    assert!(
        cpp.contains("const std::vector<ll>& a = cpl::at(rows, i);"),
        "{cpp}"
    );
}

#[test]
fn a_str_grows_in_place_when_appended_to() {
    let body =
        body("let s: Str = read()\nvar t = \"\"\nfor c in s: t += c\nprint(t, s[1..], s[..=1])\n");
    assert!(body.contains("t.push_back((char)c)"), "{body}");
    assert!(!body.contains("Capture"), "{body}");
}

#[test]
fn narrowing_a_record_array_reads_the_source_in_place() {
    let body = body(concat!(
        "domain E = 0..3\n",
        "let edges: [E](u: Int, v: Int, w: Int) = read()\n",
        "fn ends<R: (u: Int, v: Int, ..)>(es: [E]R) -> Int = es[E(0)].u + es[E(2)].v\n",
        "print(ends(edges))\n",
    ));
    assert!(!body.contains("= edges;"), "{body}");
    assert!(body.contains("& t"), "{body}");
}

#[test]
fn a_param_returned_in_a_struct_is_taken_by_value_and_moved_in() {
    let cpp = cpp("struct Pack(xs: List<Int>, ys: List<Int>)
fn pack(xs: List<Int>, ys: List<Int>) -> lends Pack:
    Pack(xs, ys)
fn twice(xs: List<Int>) -> lends Pack = Pack(xs, xs)
fn make(n: Int) -> Pack:
    let a = [i for i in 0..n]
    let b = [i * i for i in 0..n]
    return pack(a, b)
let n: 1..=100 = read()
let p = make(n)
let q = twice(p.xs)
print(p.xs.len + p.ys.len, q.xs.len)
");
    assert!(
        cpp.contains("Pack pack(std::vector<ll> xs, std::vector<ll> ys) {"),
        "{cpp}"
    );
    assert!(
        cpp.contains("return Pack{std::move(xs), std::move(ys)};"),
        "{cpp}"
    );
    assert!(
        cpp.contains("return ::prog::pack(std::move(a), std::move(b));"),
        "{cpp}"
    );
    assert!(
        cpp.contains("Pack twice(const std::vector<ll>& xs) {"),
        "named twice: copied, not sunk: {cpp}"
    );
    assert!(
        cpp.contains("::prog::twice(p.xs)"),
        "a global is not moved from: {cpp}"
    );
}
#[test]
fn an_array_built_by_a_comprehension_moves_into_its_variable() {
    let cpp = cpp("let n: 1..=10 = read()
domain V = 0..n
var d: [V]Int = [v.ord for v in V]
d[V.first!] += 5
print(d.sum())
");
    assert!(cpp.contains("d = std::move(t"), "{cpp}");
}

#[test]
fn map_keys_and_values_iterate_without_copying() {
    let out = body(concat!(
        "var m: HashMap<Int, Int> = [1: 10, 2: 20]\n",
        "var s = 0\n",
        "for k in m.keys: s += k\n",
        "print(s + m.values.sum())\n",
    ));
    assert!(
        !out.contains("std::vector"),
        "Map.keys and Map.values loops/folds must not allocate a vector: {out}"
    );
}

#[test]
fn mapped_let_read_whole_maps_once_not_per_read() {
    let out = cpp("let s: Str = read()
let blocks = s.runs().map(r => r.len)
var t = 0
for k in 0..blocks.len: t += blocks[k] + blocks.first! + blocks.last!
print(t)
");
    let solve = out.split("void solve()").nth(1).expect("solve() emitted");
    let body = solve.split("for (ll k").nth(1).expect("loop emitted");
    assert!(
        !body.contains("push_back"),
        "`.first` on a mapped `let` must not rebuild the mapped list in the loop:\n{solve}"
    );
}

#[test]
fn first_gt_and_last_lt_on_views_do_not_copy() {
    let out = cpp("use std.seq.{first_gt, last_lt}
let xs = [1, 2, 3, 3, 5, 7, 9]
let a = xs[1..6].first_gt(3)
let b = xs[1..6].last_lt(3)
let ps = [(1, 10), (2, 20), (3, 30), (3, 40), (5, 50)]
let c = ps[1..5].first_gt(3, by: p => p.0)
let d = ps[1..5].last_lt(3, by: p => p.0)
let s = \"abcde\"
let e = s[1..4].first_gt('c')
let f = s[1..4].last_lt('c')
print(a ?? -1, b ?? -1, c ?? -1, d ?? -1, e ?? -1, f ?? -1)
");
    let solve = out.split("void solve()").nth(1).expect("solve() emitted");
    assert!(
        !solve.contains("cpl::to_list(") && !solve.contains("cpl::to_str("),
        "first_gt/last_lt on slice views must not materialize a copy:\n{solve}"
    );
    assert!(
        solve.contains("cpl::list_slice(") && solve.contains("cpl::str_slice("),
        "first_gt/last_lt on slice views should search the O(1) view directly:\n{solve}"
    );
}

#[test]
fn seq_view_ops_pass_views_without_to_list_copy() {
    let out = cpp(
        "use std.seq.{merge_sorted, zip, compress, inversions, resized}
let xs = [1, 3, 5, 2, 4, 6]
let m = merge_sorted(xs[0..3], xs[3..6])
let z = zip(xs[0..3], xs[3..6])
let (C, c) = compress(xs[1..5])
let inv = inversions(xs[1..5])
let r = xs[1..4].resized(5, 0)
print(m.len, z.len, C.count, inv, r.len)
",
    );
    let solve = out.split("void solve()").nth(1).expect("solve() emitted");
    assert!(
        !solve.contains("cpl::to_list("),
        "generalized std.seq ops on slice views must not emit cpl::to_list:\n{solve}"
    );
}

#[test]
fn seq_take_is_o1_move_without_copy() {
    let out = cpp("use std.seq
var a = [1, 2, 3]
let b = seq.take_all(mut a)
print(a.len, b.len)
");
    let take_fn = out
        .split("inline std::vector<ll> m1_take_")
        .nth(1)
        .and_then(|s| s.split("void solve()").next())
        .expect("seq.take emitted");
    assert!(
        take_fn.contains("std::exchange("),
        "seq.take must move via std::exchange (O(1)):\n{take_fn}"
    );
    assert!(
        out.contains("#include <utility>\n"),
        "std::exchange needs <utility>; libstdc++ does not always pull it in:\n{out}"
    );
    assert!(
        !take_fn.contains(".resize(") && !take_fn.contains("cpl::copy("),
        "seq.take must not copy/truncate:\n{take_fn}"
    );
}

#[test]
fn a_local_read_for_the_last_time_moves_into_its_place() {
    let src = |after: &str| {
        format!(
            "let n: Int = read()\nvar dp = [0 for _ in 0..n]\nfor _ in 0..n:\n    let next = [dp[i] + i for i in 0..n]\n    dp = next\n{after}print(dp[0])\n"
        )
    };
    let moved = cpp(&src(""));
    assert!(moved.contains("dp = std::move(next);"), "{moved}");
    let read = cpp(&src("").replace(
        "    dp = next\n",
        "    dp = next.clone()\n    print(next[0])\n",
    ));
    assert!(
        !read.contains("dp = std::move(next);"),
        "next is read after the store: {read}"
    );
}

#[test]
fn a_field_a_literal_reads_before_it_is_reset_moves_in() {
    let src = |reset: &str| {
        format!(
            "struct Box:\n    xs: List<Int> = []\n    k: Int = 0\n\n    fn take(mut self) -> Box:\n        let out = Box(xs, k)\n{reset}        out\n\nvar b = Box(xs: [1, 2, 3], k: 2)\nlet c = b.take()\nprint(c.xs.len, b.xs.len)\n"
        )
    };
    let moved = cpp(&src("        xs = []\n        k = 0\n"));
    assert!(moved.contains("Box{std::move(self.xs), self.k}"), "{moved}");
    let kept = cpp(&src("        k = 0\n"));
    assert!(
        !kept.contains("std::move(self.xs)"),
        "xs is never set again: {kept}"
    );
}

#[test]
fn a_dead_field_moves_into_a_literal_whatever_its_type() {
    let src = |reset: &str| {
        format!(
            "struct Tree:\n    xs: List<Int>\n    fn new() -> Tree = Tree(xs: [])\n\nstruct Bag:\n    ids: List<-2_147_483_648..=2_147_483_647>\n    t: Tree\n\n    fn take(mut self) -> Bag:\n        let out = Bag(ids, t)\n{reset}        out\n\nvar b = Bag(ids: [1, 2], t: Tree.new())\nlet c = b.take()\nprint(c.ids.len + c.t.xs.len, b.ids.len)\n"
        )
    };
    let moved = cpp(&src("        ids = []\n        t = Tree.new()\n"));
    assert!(
        moved.contains("Bag{std::move(self.ids), std::move(self.t)}"),
        "{moved}"
    );
    let read = cpp(concat!(
        "struct Tree:\n    xs: List<Int>\n    fn new() -> Tree = Tree(xs: [])\n",
        "struct Bag:\n    ids: List<-2_147_483_648..=2_147_483_647>\n    t: Tree\n",
        "var ids: List<-2_147_483_648..=2_147_483_647> = [1, 2]\n",
        "var t = Tree.new()\n",
        "fn peek() -> Int = ids.len + t.xs.len\n",
        "let out = Bag(ids, t)\n",
        "print(peek())\n",
        "ids = []\n",
        "t = Tree.new()\n",
        "print(out.ids.len + out.t.xs.len, peek())\n",
    ));
    assert!(
        read.contains("Bag{ids, t}"),
        "peek reads both before the reset: {read}"
    );
}

#[test]
fn a_get_of_a_stored_field_or_an_element_of_one_lends_it() {
    let out = cpp("struct H:
    xs: List<Int>
    g: [0..2]List<Int>
    @get pub fn items(self) -> List<Int> = xs
    @get pub fn row(self) -> List<Int> = g[1]
    @get pub fn fresh(self) -> List<Int> = [x + 1 for x in xs]
let n: Int = read()
let h = H([i for i in 0..n], [[1], [2]])
var t = 0
for k in 0..n: t += h.items[k] + h.row[0] + h.fresh[k]
for v in H([n], [[1], [2]]).items: t += v
print(t)
");
    assert!(
        out.contains("inline const std::vector<ll>& H_items(const H& self)"),
        "{out}"
    );
    assert!(
        out.contains("inline const std::vector<ll>& H_row(const H& self)"),
        "{out}"
    );
    assert!(
        out.contains("inline std::vector<ll> H_fresh(const H& self)"),
        "{out}"
    );
}

#[test]
fn a_returned_literal_moves_the_parts_of_a_dead_local() {
    let out = cpp(concat!(
        "struct S: pub a: List<Int>\n",
        "struct T:\n    pub a: List<Int>\n    pub n: Int\n",
        "fn f() -> S:\n    var s = S([0 for _ in 0..10])\n    s.a.push(1)\n    S(s.a)\n",
        "fn g() -> S:\n    let st = [(1, [0 for _ in 0..10])]\n    S(st[0].1)\n",
        "fn h() -> List<Int>:\n    var s = S([0 for _ in 0..10])\n    s.a.push(1)\n    s.a\n",
        "fn kept(s: S) -> S = S(s.a)\n",
        "struct U:\n    pub a: List<Int>\n    pub b: List<Int>\n",
        "fn both() -> U:\n    var u = U([0 for _ in 0..10], [1 for _ in 0..10])\n    u.a[0] = 5\n    U(u.a, u.b)\n",
        "fn twice() -> T:\n    var s = S([0 for _ in 0..10])\n    s.a.push(1)\n    T(s.a, s.a.len)\n",
        "print(f().a.len, g().a.len, h().len, kept(f()).a.len, twice().a.len + twice().n, both().b.len)\n",
    ));
    assert!(out.contains("return S{std::move(s.a)};"), "{out}");
    assert!(out.contains("return S{std::move(std::get<1>("), "{out}");
    assert!(out.contains("return std::move(s.a);"), "{out}");
    assert!(
        out.contains("return U{std::move(u.a), std::move(u.b)};"),
        "{out}"
    );
    assert!(
        out.contains("return S{s.a};"),
        "a parameter is the caller's: {out}"
    );
    assert!(out.contains("T{s.a, "), "another item reads `s`: {out}");
}

const HANDED_OUT: &str = "struct S(xs: List<Int>, k: Int)
fn tail_some(n: Int) -> ?(Int, List<Int>):
    var x = [n]
    x.push(1)
    Some((n, x))
fn early_some(n: Int) -> ?(Int, List<Int>):
    var x = [n]
    x.push(1)
    if n > 5: return Some((n, x))
    None
fn tail_struct(n: Int) -> S:
    var xs = [n]
    xs.push(2)
    S(xs, n)
fn tail_tuple(n: Int) -> (List<Int>, List<Int>):
    var p = [n]
    var q = [n, n]
    p.push(1)
    (p, q)
let n: Int = read()
print(tail_some(n)!.1, early_some(n + 6)!.1, tail_struct(n).xs, tail_tuple(n).0)
let (y, z) = [1, 2, 3].partition(v => v > n)
print(y, z)
print(yes([2, 4].partition(x => x % 2 == 0) == ([2, 4], [])))
";

/// The C++ lines that build a tuple or struct of lists from names.
fn built_from_names(cpp: &str) -> Vec<&str> {
    cpp.lines()
        .filter(|l| l.contains("std::vector<ll>") && (l.contains("{") && l.contains("}")))
        .filter(|l| l.contains("return ") || l.contains("> t"))
        .filter(|l| l.contains("std::tuple<") || l.contains(" S{"))
        .collect()
}

#[test]
fn a_return_moves_each_local_it_is_the_last_use_of() {
    let cpp = cpp(HANDED_OUT);
    // `Some((n, x))`, as a tail and as an early `return`.
    assert_eq!(cpp.matches("{n_2, std::move(x)}").count(), 2, "{cpp}");
    assert!(cpp.contains("return S{std::move(xs), n_2};"), "{cpp}");
    assert!(cpp.contains("{std::move(p), std::move(q)};"), "{cpp}");
}

#[test]
fn an_inlined_block_moves_its_own_lets_into_the_tuple_it_gives() {
    let cpp = cpp(HANDED_OUT);
    let built = built_from_names(&cpp);
    assert!(!built.is_empty(), "{cpp}");
    for line in built {
        let items = line.rsplit_once('{').map_or("", |(_, rest)| rest);
        for item in items.trim_end_matches(['}', ')', ';']).split(", ") {
            assert!(
                item.starts_with("std::move(") || !item.starts_with('t') || item == "true",
                "{line}"
            );
        }
    }
}

#[test]
fn explain_lists_no_copy_for_a_value_moved_out() {
    let copies = crate::explain_copies(&super::lower(HANDED_OUT));
    assert!(copies.is_empty(), "{copies:?}");
}

#[test]
fn a_local_dead_after_its_if_moves_into_what_a_branch_stores() {
    let out = cpp(concat!(
        "struct H:\n    pub var xs: List<Int>\n",
        "var best: ?List<Int> = None\n",
        "var h = H([])\n",
        "for i in 0..4:\n",
        "    let w = [i, i]\n",
        "    if i % 2 == 0: best = w\n",
        "    else:\n",
        "        let v = [i]\n",
        "        if i == 3: h.xs = v\n",
        "print(best!, h.xs)\n",
    ));
    assert!(
        out.contains("best = std::optional<std::vector<ll>>(std::move(w));"),
        "{out}"
    );
    assert!(out.contains("h.xs = std::move(v);"), "{out}");
}

#[test]
fn a_local_dead_after_a_literal_moves_into_it() {
    let out = cpp(concat!(
        "struct W:\n    var n: Int\n    vals: List<Int>\n",
        "fn make(a: List<Int>) -> W:\n",
        "    let vals = [x * 2 for x in a]\n",
        "    let n = vals.len\n",
        "    var w = W(n, vals)\n",
        "    w.n += w.vals[0]\n",
        "    w\n",
        "let a: List<Int> = read()\n",
        "print(make(a).n)\n",
    ));
    assert!(out.contains("W{n, std::move(vals)}"), "{out}");
}

#[test]
fn an_if_let_over_a_fresh_optional_moves_its_payload() {
    let out = cpp(concat!(
        "fn f(n: Int) -> ?List<Int> = if n > 0: Some([n]) else: None\n",
        "fn g(n: Int) -> ?List<Int>:\n",
        "    if let got = f(n): return Some(got)\n",
        "    None\n",
        "print(yes(g(read()) == None))\n",
    ));
    assert!(
        out.contains("std::vector<ll> got = std::move(*t0);"),
        "{out}"
    );
    assert!(!out.contains("& got"), "{out}");
}

#[test]
fn a_param_dead_after_its_sink_moves_past_a_closure_that_does_not_hold_it() {
    let out = cpp(concat!(
        "fn join(a: List<(Int, Int)>, b: List<(Int, Int)>) -> List<(Int, Int)>:\n",
        "    var all = a\n",
        "    for x in b:\n",
        "        if let i = (0..all.len).find(i => all[i].0 == x.0): all[i].1 += x.1\n",
        "        else: all.push(x)\n",
        "    all\n",
        "fn held(a: List<Int>) -> Int:\n",
        "    let g = () => a.len\n",
        "    var all = a\n",
        "    all.push(1)\n",
        "    g() + all.len\n",
        "let n: Int = read()\n",
        "let a = [(i, i) for i in 0..n]\n",
        "print(join(a, a).len, held([n]))\n",
    ));
    assert_eq!(
        out.matches("all = std::move(a").count(),
        1,
        "join's param moves; held's, which a closure made first holds, is copied:\n{out}"
    );
}

#[test]
fn a_list_sorted_by_a_key_at_its_last_use_moves_into_the_sort() {
    let out = cpp(concat!(
        "let n: 2..=10 = read()\n",
        "let xs: [n](t: Int, x: Int) = read()\n",
        "print(xs.sorted(by: o => o.t).first!.x)\n",
    ));
    assert!(out.contains("= std::move(xs);"), "{out}");
}

#[test]
fn a_param_copied_whole_into_a_var_is_taken_by_value_and_moved_in() {
    let out = cpp(concat!(
        "fn top(xs: List<Int>) -> List<Int>:\n",
        "    var ys = List(xs)\n",
        "    ys.sort()\n",
        "    ys\n",
        "print(top([read<Int>(), 1]))\n",
    ));
    assert!(out.contains("(std::vector<ll> xs)"), "{out}");
    assert!(out.contains("= std::move(xs);"), "{out}");
}

#[test]
fn a_local_passed_at_its_last_read_moves_into_a_var_param() {
    let out = cpp(concat!(
        "fn f(var xs: List<Int>, d: Int):\n",
        "    if d == 0:\n",
        "        print(xs.len)\n",
        "        return\n",
        "    xs.push(d)\n",
        "    let ys = xs.filter(x => x > 0).to_list()\n",
        "    f(ys, d - 1)\n",
        "f([read<Int>()], 3)\n",
    ));
    assert!(out.contains("(std::move(ys), "), "{out}");
}

#[test]
fn a_param_a_literal_stores_at_its_last_read_is_sunk_and_moved_in() {
    let out = cpp(concat!(
        "struct R:\n",
        "    var n: Int\n",
        "    vals: List<Int>\n",
        "    keys: Range<Int>\n",
        "    pub fn new(a: List<Int>) -> R:\n",
        "        let n = a.len\n",
        "        let keys = a.keys\n",
        "        var q = R(n, a, keys)\n",
        "        q.n += 1\n",
        "        q\n",
        "struct W(xs: List<Int>)\n",
        "fn kept(a: List<Int>) -> Int:\n",
        "    let w = W(a)\n",
        "    w.xs.len + a.len\n",
        "struct S(sa: List<Int>, r: R)\n",
        "fn build(m: Int) -> S:\n",
        "    let sa = [i for i in 0..m]\n",
        "    let lcp = [i * 2 for i in 0..m]\n",
        "    S(sa, R(lcp))\n",
        "let m: Int = read()\n",
        "let s = build(m)\n",
        "print(s.sa.len + s.r.vals.len + s.r.n, kept(s.sa))\n",
    ));
    assert!(out.contains("R R_new(std::vector<ll> a)"), "{out}");
    assert!(out.contains("R q = R{n, std::move(a)"), "{out}");
    assert!(out.contains("R_new(std::move(lcp))"), "{out}");
    assert!(
        out.contains("ll kept(const std::vector<ll>& a)"),
        "read after the literal, so lent and copied: {out}"
    );
}

#[test]
fn a_let_of_a_lent_node_is_a_reference_unless_its_slot_is_written_while_it_lives() {
    let cpp = cpp(concat!(
        "use std.marks.{Arena, Id}\n",
        "struct N:\n    var a: Int\n    var big: List<Int>\n",
        "fn read(ar: Arena<N>, i: Id<N>) -> Int:\n    let n = ar.get(i)\n    n.a + n.big.len\n",
        "fn write(mut ar: Arena<N>, i: Id<N>) -> Int:\n",
        "    let m = ar.get(i)\n    ar.at(i).a = 7\n    m.a\n",
        "var ar = Arena<N>()\n",
        "let i = ar.alloc(N(1, [2]))\n",
        "print(read(ar, i), write(mut ar, i), ar.at(i).a + ar.at(i).big.len)\n",
    ));
    assert!(cpp.contains("const N& n = Arena_"), "{cpp}");
    assert!(
        cpp.contains("N m = Arena_"),
        "the slot changes while `m` lives: {cpp}"
    );
    assert!(cpp.contains("ar._xs, ll(i.raw)).a, "), "{cpp}");
    assert!(
        !cpp.contains("-> N {"),
        "a read through `at` builds no node: {cpp}"
    );
}

#[test]
fn an_optional_struct_param_the_fn_only_reads_is_lent_not_copied() {
    let cpp = cpp(concat!(
        "struct H(pre: List<Int>):\n",
        "    fn at(self, i: Int, of: ?H = None) -> Int:\n",
        "        if let o = of: return o.pre[i]\n",
        "        self.pre[i]\n",
        "let h = H([1, 2, 3])\n",
        "let h2 = H([4, 5, 6])\n",
        "print(h.at(0), h.at(1, of: h2))\n",
    ));
    assert!(
        cpp.contains("cpl::Lent<H> of)") && cpp.contains("cpl::Lent<H>& t0 = of;"),
        "{cpp}"
    );
    assert!(
        cpp.contains("cpl::lend(h2)"),
        "`h2` is passed by pointer, not copied into an optional:\n{cpp}"
    );
    assert!(!cpp.contains("std::optional<H>(h2)"), "{cpp}");
}

#[test]
fn a_mut_self_call_through_a_lend_runs_on_the_slot() {
    let cpp = cpp(concat!(
        "use std.marks.{Arena, Id}\n",
        "struct C:\n    var n: Int\n    fn bump(mut self, by: Int):\n        n += by\n",
        "struct N:\n    var kids: List<Int>\n    var c: C\n",
        "fn grow(mut ar: Arena<N>, i: Id<N>):\n",
        "    ar.at(i).kids.push(3)\n    ar[i].c.bump(2)\n",
        "var ar = Arena<N>()\n",
        "let i = ar.alloc(N([1], C(0)))\n",
        "grow(mut ar, i)\n",
        "print(ar.at(i).kids, ar.at(i).c.n)\n",
    ));
    assert!(
        cpp.contains("ar_2._xs, ll(i_2.raw)).kids.push_back(3);"),
        "{cpp}"
    );
    assert!(
        cpp.contains("C_bump(cpl::at(ar_2._xs, ll(i_2.raw)).c, 2);"),
        "{cpp}"
    );
    assert!(!cpp.contains("N t"), "no node is copied: {cpp}");
}
