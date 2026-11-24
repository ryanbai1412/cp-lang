//! How range loops are written in C++.

use super::cpp;

#[test]
fn an_inclusive_loop_to_a_bound_below_int_max_is_a_plain_loop() {
    let open = cpp("let n: Int = read()\nfor i in 0..=n:\n    print(i)\n");
    assert!(open.contains("i_go"), "{open}");
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
