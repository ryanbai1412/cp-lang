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
        cpp.contains("const std::optional<std::vector<ll>>& t0 = end;"),
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
