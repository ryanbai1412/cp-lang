//! How range loops are written in C++.

use super::cpp;
use crate::tests::body;

#[test]
fn an_inclusive_loop_to_a_bound_below_int_max_is_a_plain_loop() {
    let open = cpp("let n: Int = read()\nfor i in 0..=n:\n    print(i)\n");
    assert!(open.contains("i_go"), "{open}");
    assert!(
        open.contains("i != n && (++i, true)"),
        "a counted step to a last value: {open}"
    );
    let len = cpp("let xs = [1, 2, 3]\nfor i in 0..=xs.len:\n    print(i)\n");
    assert!(len.contains("i <= "), "{len}");
    assert!(!len.contains("i_go"), "{len}");
    let tuple = cpp(
        "domain D = 0..5\nlet n: Int = read()\nlet (k, m) = (n, D.count)\nfor i in 1..=m - 1:\n    print(i, k)\n",
    );
    assert!(!tuple.contains("i_go"), "{tuple}");
    let quotient = cpp(
        "let (n, m): (1..=10, 2..=3000) = read()\nfor j in 1..n:\n    for d in 1..=m / (j + 1):\n        print(d)\n",
    );
    assert!(!quotient.contains("d_go"), "{quotient}");
    let wrapping = cpp(
        "let m: 2..=3000 = read()\nunchecked:\n    for j in 1..m:\n        for d in 1..=m / (j + 1):\n            print(d * (j + 1))\n",
    );
    assert!(
        !wrapping.contains("d_go") && !wrapping.contains("ediv"),
        "{wrapping}"
    );
    assert!(
        !wrapping.contains("uint64_t(j)"),
        "a sum that can't wrap is plain: {wrapping}"
    );
}

#[test]
fn a_map_scanned_only_into_a_list_sorted_by_value_is_a_hash_table() {
    let source = concat!(
        "let n: 1..=1000 = read()\n",
        "let a: [0..n]Int = read()\n",
        "var freq: Map<Int, Int> = Map()\n",
        "for x in a: freq[x] += 1\n",
        "let odd = [cnt for (x, cnt) in freq.items if x % 2 == 1].sorted(by: c => -c)\n",
        "print(odd)\n",
    );
    let hashed = cpp(source);
    assert!(!hashed.contains("std::map<ll, ll> freq"), "{hashed}");
    let desc = cpp(&source.replace("sorted(by: c => -c)", "sorted(by: c => desc(c))"));
    assert!(!desc.contains("std::map<ll, ll> freq"), "{desc}");
    let ordered = cpp(&source.replace("sorted(by: c => -c)", "rev"));
    assert!(ordered.contains("std::map<ll, ll> freq"), "{ordered}");
}

#[test]
fn explain_names_each_table_stored_as_a_hash_table() {
    let source = concat!(
        "let n: 1..=1000 = read()\n",
        "let a: [0..n]Int = read()\n",
        "var freq: Map<Int, Int> = Map()\n",
        "for x in a: freq[x] += 1\n",
        "print(freq.get(3) ?? 0)\n",
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let report = crate::explain::report("t.cp", source, &program);
    assert!(report.contains("[hashed] Map<Int, _>"), "{report}");
}

#[test]
fn a_stepped_range_walked_by_a_loop_or_generator_is_a_counted_loop() {
    let out = cpp("let (n, g): (1..=100_000_000, 1..=100) = read()
print(sum(a for a in (g..=n).step(g) if a % 3 == 0))
var t = 0
for a in (0..n).step(g): t += a
print(t)
");
    assert!(
        !out.contains("steps.push_back"),
        "`.step(g)` must not list the range first:\n{out}"
    );
    assert!(out.contains("cpl::steps(a, n, g, true)"), "{out}");
}

#[test]
fn a_reversed_range_find_stops_at_the_first_hit() {
    let out = cpp("var xs = [1, 2, 3, 4]
xs.push(5)
print((0..xs.len).rev.find(i => xs[i] > 2)!)
");
    assert!(
        !out.contains("-> std::vector<ll> {"),
        "`(0..n).rev.find(p)` must not list the range first:\n{out}"
    );
}

#[test]
fn a_grid_read_and_written_at_product_positions_indexes_the_cell_by_ordinal() {
    // `(i / n, i % n)` per position put back together by the grid is `i`.
    let out = cpp(concat!(
        "let n: 1..=10 = read()\n",
        "domain City = 0..n\n",
        "domain Corner = 0..4\n",
        "type Airport = (City, Corner)\n",
        "var w = [Airport, Airport] (u, v) => u.0.ord + v.1.ord\n",
        "for i in Airport:\n",
        "    for j in Airport: w[i, j] = min(w[i, j], w[j, i] + 1)\n",
        "print(w[(City.first!, Corner.first!), (City.last!, Corner.last!)])\n",
    ));
    assert!(out.contains("w.d[(size_t)("), "{out}");
    assert!(
        !out.contains("emod((ll)i"),
        "no position is taken apart: {out}"
    );
}

#[test]
fn a_callback_loop_that_never_reads_its_item_marks_it_unused() {
    // `deg` counts the arcs `out` yields; g++ -Wextra must stay quiet.
    let out = cpp(concat!(
        "let n: Int = read()\n",
        "domain V = 0..n\n",
        "let es: [V](u: V, v: V) = [(u: v, v: V((v.ord + 1) % n)) for v in V]\n",
        "print(Graph.undirected(es).only(keep: v => v.ord != 1).deg(V(0)))\n",
    ));
    assert!(
        out.contains("[&]([[maybe_unused]] const std::tuple<"),
        "{out}"
    );
}

#[test]
fn a_max_plus_row_is_one_branch_free_pass_on_an_avx2_judge_only() {
    let program = crate::tests::lower(concat!(
        "let n: 1..=8 = read()\n",
        "domain S = 0..n\n",
        "type Row = [S]?Int\n",
        "fn keep(acc: Row, row: Row, base: Int) -> Row:\n",
        "    var c = acc\n",
        "    for l in S:\n",
        "        if let w = row[l]: c[l] max= Some(base + w)\n",
        "    c\n",
        "let row: Row = [S] s => Some(Int(s))\n",
        "print(keep(row, row, 3)[S.first!] ?? -1)\n",
    ));
    let avx2 = crate::Target {
        cpu: vec!["avx2".into()],
        ..crate::Target::default()
    };
    let fast = crate::emit_for(&program, "t.cp", None, &avx2);
    assert!(fast.contains("cpl::keep_max_plus(c.data(), "), "{fast}");
    let plain = crate::emit_for(&program, "t.cp", None, &crate::Target::default());
    assert!(!plain.contains("keep_max_plus"), "{plain}");
}

#[test]
fn a_grids_keys_walk_one_loop_per_axis_and_divide_nothing() {
    let source = concat!(
        "let n: 1..=1500 = read()\n",
        "domain Row = 0..n\n",
        "domain Col = 0..n\n",
        "let a: [Row, Col]1..=9 = read()\n",
        "var s = 0\n",
        "for p in a.keys:\n",
        "    if p.1.ord == 2: continue\n",
        "    let (r, c) = p\n",
        "    s += p.0.ord * a[p] + r.ord - c.ord\n",
        "print(s)\n",
    );
    let body = body(source);
    assert!(
        body.contains("< Row_len; t") && body.contains("< Col_len; t"),
        "{body}"
    );
    assert!(
        !body.contains("cpl::divn") && !body.contains("cpl::ediv"),
        "{body}"
    );
    assert!(
        !body.contains("cpl::modn") && !body.contains("cpl::emod") && !body.contains('%'),
        "{body}"
    );
}

#[test]
fn a_ranges_table_is_indexed_by_plain_index_math() {
    let body = body(concat!(
        "let n: 1..=3000 = read()\ndomain I = 1..=n\nlet a: [I]Int = read()\n",
        "var lead: [I.ranges]Int = 0\nvar cost: [I.ranges]Int = 0\n",
        "for r in I.ranges:\n",
        "    lead[r] = if r.len == 1: a[r.first] else: max(\n",
        "        a[r.first] - lead[r.drop(1)],\n        a[r.last] - lead[r.drop_last(1)],\n    )\n",
        "    if r.len >= 2:\n",
        "        cost[r] = min(cost[x] + cost[y] for (x, y) in r.splits)!\n",
        "print(lead[I.range], cost[I.range])\n",
    ));
    // End-major cells, ends already in order: no swap per read.
    assert!(body.contains("cpl::tri_idx("), "{body}");
    assert!(!body.contains("cpl::sym_idx("), "{body}");
    // `r` is a key, so its own cell has no empty check; `drop`, `drop_last`
    // and the split pieces read the ends without copying a `Range`. The
    // pieces' checks go too: the cut walks `r.drop(1)`, so both are proven
    // non-empty.
    let checks = body.matches("an empty range has no cell").count();
    assert_eq!(checks, 4, "the two drops and the two `I.range`s: {body}");
    assert!(!body.contains("cpl::Range<int> piece"), "{body}");
    assert!(!body.contains("cpl::Range<int> range = r"), "{body}");
    // `r.splits` runs in place as the loop over its cuts: no callback form.
    assert!(!body.contains("cpl_each_"), "{body}");
    assert!(body.contains("for (int p = "), "{body}");
    // `min` over the splits starts from its identity: no first-value branch.
    assert!(body.contains("9223372036854775807LL"), "{body}");
}

#[test]
fn a_split_of_a_cut_walking_a_drop_has_no_empty_check() {
    // A user fn shaped like `splits`: the bounds of the loop over the cuts
    // prove both pieces non-empty, whatever the fn is called.
    let source = concat!(
        "fn halves<D>(r: Range<D>) -> yields (Range<D>, Range<D>):\n",
        "    for p in r.drop(1):\n",
        "        let m = (p..p).l\n",
        "        yield (r.l..m, m..r.r)\n",
        "let n: 1..=3000 = read()\ndomain I = 1..=n\nlet a: [I]Int = read()\n",
        "var cost: [I.ranges]Int = 0\n",
        "for r in I.ranges:\n",
        "    if r.len >= 2:\n",
        "        cost[r] = min(cost[x] + cost[y] + a[r.first] for (x, y) in halves(r))!\n",
        "print(cost[I.range])\n",
    );
    let body = body(source);
    let checks = body.matches("an empty range has no cell").count();
    assert_eq!(checks, 1, "only `I.range`'s: {body}");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let report = crate::explain::report("t.cp", source, &program);
    assert_eq!(
        report.matches(crate::explain::PROVEN).count(),
        2,
        "{report}"
    );
}

#[test]
fn a_split_whose_cut_can_reach_an_end_keeps_its_check() {
    // The cut walks all of `r`, so `r.l..m` is empty at the first one.
    let body = body(concat!(
        "fn halves<D>(r: Range<D>) -> yields (Range<D>, Range<D>):\n",
        "    for p in r:\n",
        "        let m = (p..p).l\n",
        "        yield (r.l..m, m..r.r)\n",
        "let n: 1..=3000 = read()\ndomain I = 1..=n\n",
        "var cost: [I.ranges]Int = 0\n",
        "for r in I.ranges:\n",
        "    if r.len >= 2:\n",
        "        cost[r] = min(cost[x] + cost[y] for (x, y) in halves(r))!\n",
        "print(cost[I.range])\n",
    ));
    assert_eq!(
        body.matches("an empty range has no cell").count(),
        2,
        "{body}"
    );
}

#[test]
fn a_cache_fill_is_the_unlikely_branch() {
    let out = cpp("struct Memo:\n    @cache t: List<Int>\n\
         \n    fn grow(self, n: Int):\n        while t.len <= n: t.push(t.len * t.len)\n\
         \n    fn at(self, n: Int) -> Int:\n        if n >= t.len: self.grow(n)\n        t[n]\n\
         \nlet m = Memo([])\nlet n: 0..=100 = read()\nif n > 5: print(n)\nprint(m.at(n))\n");
    let fill = out
        .lines()
        .find(|l| l.contains("if (") && l.contains("self.t.size()"));
    assert!(
        fill.is_some_and(|l| l.contains("if (__builtin_expect(")),
        "{out}"
    );
    assert_eq!(out.matches("__builtin_expect(n").count(), 1, "{out}");
}

#[test]
fn comb_inv_in_a_loop_is_one_load_with_no_mulmod() {
    let out = cpp(
        "type Z = Mod<1_000_000_007>\nlet c = Comb<Z>()\nlet n: 1..=5000 = read()\n\
         var acc = Z(0)\nfor i in 1..=n: acc += c.inv(i)\nprint(acc)\n",
    );
    let (at, _) = out
        .match_indices("_inv(Comb")
        .find(|(i, _)| !out[..*i].ends_with("_grow"))
        .expect("Comb.inv is emitted");
    let inv = &out[at..at + out[at..].find("\n}\n").expect("its body ends")];
    assert!(inv.contains("if (__builtin_expect("), "{inv}");
    let ret = inv.lines().find(|l| l.contains("return ")).unwrap_or("");
    assert!(
        ret.contains("self.iv[i]") || ret.contains("cpl::at(self.iv, i)"),
        "{inv}"
    );
    assert!(!inv.contains(" % ") && !inv.contains("_mul("), "{inv}");
}

#[test]
fn a_contains_test_before_its_add_walks_the_set_once() {
    let out = cpp("let n: Int = read()
var s: Set<Int> = []
var h: HashSet<Int> = []
var seen = 0
for i in 0..n:
    let x = i * 7 % 13
    if s.contains(x): seen += 1
    s.add(x)
    if h.contains(x):
        seen += 1
        if h.len > 3: return
    h.add(x)
print(seen, s.len, h.len)
");
    assert!(out.contains("if (!(s.insert(x).second)) {"), "{out}");
    assert!(!out.contains("s.count(x)"), "{out}");
    assert!(
        out.contains("h.count(x)"),
        "a body that reads the set or leaves keeps the test: {out}"
    );
}

#[test]
fn a_loop_over_the_rows_of_a_grid_walks_them_by_pointer() {
    let source = concat!(
        "let n: 1..=100 = read()\n",
        "domain S = 0..n\n",
        "fn mul(a: [S, S]Int, b: [S, S]Int) -> [S, S]Int:\n",
        "    var c: [S, S]Int = [S, S] (i, j) => 0\n",
        "    for i in S:\n",
        "        for j in S:\n",
        "            let via = a[i, j]\n",
        "            for l in S: c[i, l] max= via + b[j, l]\n",
        "    c\n",
        "let g: [S, S]Int = [S, S] (i, j) => i.ord + j.ord\n",
        "print(mul(g, g)[(S.first)!, (S.first)!])\n",
    );
    let out = cpp(source);
    assert!(
        out.contains("++, c_row += c.n[1], a_row += a.n[1]"),
        "the rows of `c` and `a` move with `i`: {out}"
    );
    assert!(out.contains("++, b_row += b.n[1]"), "{out}");
    assert!(
        out.contains("c_row[(size_t)(l)] = ") && out.contains("b_row[(size_t)(l)]"),
        "{out}"
    );
    let rebound = cpp(concat!(
        "let n: 1..=100 = read()\n",
        "domain S = 0..n\n",
        "var c: [S, S]Int = [S, S] (i, j) => 0\n",
        "for i in S:\n",
        "    c[i, (S.first)!] = 1\n",
        "    c = [S, S] (i, j) => 2\n",
        "print(c[(S.first)!, (S.first)!])\n",
    ));
    assert!(
        !rebound.contains("c_row"),
        "a grid the loop replaces: {rebound}"
    );
}
