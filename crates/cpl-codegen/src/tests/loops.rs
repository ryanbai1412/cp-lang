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
    // and the split pieces read the ends without copying a `Range`.
    let checks = body.matches("an empty range has no cell").count();
    assert_eq!(
        checks, 6,
        "the two drops, the two pieces and the two `I.range`s: {body}"
    );
    assert!(!body.contains("cpl::Range<int> piece"), "{body}");
    assert!(!body.contains("cpl::Range<int> range = r"), "{body}");
    // `min` over the splits starts from its identity: no first-value branch.
    assert!(body.contains("9223372036854775807LL"), "{body}");
}
