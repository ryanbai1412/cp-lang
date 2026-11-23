//! Values the C++ reads in place instead of copying.

use super::{body, cpp};

#[test]
fn if_let_map_get_points_into_the_map_unless_the_map_changes() {
    let lent = body(concat!(
        "var m = Map<Int, List<Int>>()\n",
        "m[0] = [1, 2, 3]\n",
        "var s = 0\n",
        "for i in 0..3:\n",
        "    if let l = m.get(0):\n",
        "        s += l.len\n",
        "print(s)\n",
    ));
    assert!(
        lent.contains("cpl::map_lend(m, 0)"),
        "expected a lent entry: {lent}"
    );
    assert!(
        !lent.contains("map_get("),
        "a read-only lookup must not copy the list: {lent}"
    );
    let copied = body(concat!(
        "var m = Map<Int, List<Int>>()\n",
        "m[0] = [1, 2, 3]\n",
        "if let l = m.get(0):\n",
        "    m[0] = []\n",
        "    print(l.len)\n",
    ));
    assert!(
        copied.contains("cpl::map_get(m, 0)"),
        "a changed map keeps the copy: {copied}"
    );
}

#[test]
fn if_let_on_a_field_reads_it_in_place_when_the_branch_copies_out() {
    let lent = cpp(concat!(
        "struct H:\n",
        "    lt: ?((Int, Int) -> Bool)\n",
        "    fn less(self, a: Int, b: Int) -> Bool:\n",
        "        if let f = self.lt: return f(a, b)\n",
        "        a < b\n",
        "let h = H(lt: None)\n",
        "let g = H(lt: Some((a, b) => a > b))\n",
        "var c = 0\n",
        "for i in 0..10:\n",
        "    if h.less(i, 5) and g.less(i, 5): c += 1\n",
        "print(c)\n",
    ));
    assert!(
        lent.contains(">& t0 = self.lt;"),
        "the tested optional must not copy the fn: {lent}"
    );
    let copied = body(concat!(
        "struct H:\n",
        "    var xs: ?List<Int>\n",
        "var h = H(xs: [1, 2])\n",
        "if let l = h.xs:\n",
        "    h.xs = None\n",
        "    print(l.len)\n",
    ));
    assert!(
        !copied.contains(">& t0 = h.xs;"),
        "a list bound out of a changed place keeps its copy: {copied}"
    );
}

#[test]
fn coalesced_map_get_points_into_the_map_unless_it_changes() {
    let lent = body(concat!(
        "var m = Map<Int, List<Int>>()\n",
        "m[0] = [1, 2, 3]\n",
        "var s = 0\n",
        "for i in 0..3:\n",
        "    let l = m.get(i) ?? [i]\n",
        "    s += l.len\n",
        "print(s)\n",
    ));
    assert!(
        lent.contains("cpl::map_lend(m, i)"),
        "expected a lent entry: {lent}"
    );
    assert!(
        !lent.contains("map_get("),
        "a read-only lookup must not copy the list: {lent}"
    );
    let copied = body(concat!(
        "var m = Map<Int, List<Int>>()\n",
        "m[0] = [1, 2, 3]\n",
        "let l = m.get(0) ?? []\n",
        "m[0] = []\n",
        "var v = m.get(1) ?? [4]\n",
        "v.push(5)\n",
        "print(l.len, v.len)\n",
    ));
    assert!(
        copied.contains("cpl::map_get(m, 0)"),
        "a changed map keeps the copy: {copied}"
    );
    assert!(
        copied.contains("cpl::map_get(m, 1)"),
        "a changed binding keeps the copy: {copied}"
    );
}

#[test]
fn for_over_map_get_or_default_walks_the_entry_unless_the_body_changes_the_map() {
    let lent = body(concat!(
        "var m = Map<Int, List<Int>>()\n",
        "m[0] = [1, 2, 3]\n",
        "var s = 0\n",
        "for k in 0..3:\n",
        "    for v in m.get(k) ?? []: s += v\n",
        "print(s)\n",
    ));
    assert!(
        lent.contains("cpl::map_lend(m, k)"),
        "expected a lent entry: {lent}"
    );
    assert!(
        !lent.contains("map_get("),
        "a read-only walk must not copy the list: {lent}"
    );
    let copied = body(concat!(
        "var m = Map<Int, List<Int>>()\n",
        "m[0] = [1, 2, 3]\n",
        "for v in m.get(0) ?? []: m[0] = [v]\n",
        "print(m[0])\n",
    ));
    assert!(
        copied.contains("cpl::map_get(m, 0)"),
        "a changed map keeps the copy: {copied}"
    );
}

#[test]
fn a_named_map_get_lends_when_every_read_takes_a_lent_entry() {
    let src = |extra: &str| {
        format!(
            concat!(
                "fn f(m: Map<Int, List<Int>>, k: Int) -> Int:\n",
                "    let o = m.get(k)\n",
                "    var s = 0\n",
                "    if let l = o: s += l.len\n",
                "    for v in o ?? [9]: s += v\n",
                "    if o.is_some: s += 1\n",
                "{}",
                "    return s\n",
                "var m = Map<Int, List<Int>>()\n",
                "m[1] = [1, 2, 3]\n",
                "print(f(m, 1), f(m, 2))\n",
            ),
            extra
        )
    };
    let lent = cpp(&src(""));
    assert!(
        lent.contains("const auto o = cpl::map_lend(m_2, k);"),
        "expected a lent entry: {lent}"
    );
    assert!(
        !lent.contains("map_get("),
        "reads that take a lent entry never copy it: {lent}"
    );
    let printed = cpp(&src("    print(o)\n"));
    assert!(
        printed.contains("cpl::map_get(m_2, k)"),
        "a read that needs the optional itself keeps the copy: {printed}"
    );
}

#[test]
fn a_map_entry_read_through_a_closure_and_a_domain_block_is_never_copied() {
    let lent = body(concat!(
        "var m = Map<Int, List<Int>>()\n",
        "m[0] = [1, 5, 9]\n",
        "var s = 0\n",
        "for hi in 0..4:\n",
        "    let l = m.get(0) ?? []\n",
        "    s += l.partition_point(x => x < hi)\n",
        "print(s)\n",
    ));
    assert!(
        lent.contains("cpl::map_lend(m, 0)") && !lent.contains("map_get("),
        "a closure that writes nothing keeps the entry lent: {lent}"
    );
    assert!(
        lent.contains("-> decltype(auto)") && !lent.contains("-> std::vector<ll> {"),
        "the block that sizes the list's domain gives a reference: {lent}"
    );
    let copied = body(concat!(
        "var m = Map<Int, List<Int>>()\n",
        "m[0] = [1]\n",
        "var s = 0\n",
        "for k in 0..2:\n",
        "    let l = m.get(0) ?? [7]\n",
        "    let grow = (x: Int) => m[x + 1] = [x]\n",
        "    grow(k)\n",
        "    s += l.len + m.len\n",
        "print(s)\n",
    ));
    assert!(
        copied.contains("cpl::map_get(m, 0)"),
        "a closure that writes the map keeps the copy: {copied}"
    );
}

#[test]
fn a_map_entry_passed_to_a_fn_is_read_in_place_unless_the_fn_writes_the_map() {
    let read = body(concat!(
        "fn summed(xs: List<Int>) -> Int:\n",
        "    var s = 0\n",
        "    for x in xs: s += x\n",
        "    s\n",
        "var m = Map<Int, List<Int>>()\n",
        "m[1] = [2]\n",
        "print(summed(m.get(1) ?? []))\n",
    ));
    assert!(
        read.contains("cpl::map_or(m, 1, ") && !read.contains("map_get("),
        "the entry or the default is passed by reference: {read}"
    );
    let written = body(concat!(
        "var m = Map<Int, List<Int>>()\n",
        "fn bumped(xs: List<Int>) -> Int:\n",
        "    m[9] = [1]\n",
        "    xs.len\n",
        "m[1] = [2]\n",
        "print(bumped(m.get(1) ?? []))\n",
    ));
    assert!(
        written.contains("cpl::map_get(m, 1)") && !written.contains("map_or("),
        "a fn that writes the map gets a copy: {written}"
    );
}

#[test]
fn an_entry_stored_back_longer_is_moved_out_unless_the_rest_reads_the_map() {
    let moved = body(concat!(
        "let masks = [3, 1, 3]\n",
        "var h = Map<Int, List<Int>>()\n",
        "for v in 0..3:\n",
        "    h[masks[v]] = (h.get(masks[v]) ?? []) + [v]\n",
        "print(h.len)\n",
    ));
    assert!(
        moved.contains("cpl::map_move_or(h, "),
        "the entry is moved, not copied: {moved}"
    );
    assert!(!moved.contains("map_get("), "no copy of the entry: {moved}");
    let copied = body(concat!(
        "var h = Map<Int, List<Int>>()\n",
        "h[1] = [1]\n",
        "for v in 0..3:\n",
        "    h[v] = (h.get(v) ?? []) + h[1]\n",
        "print(h.len)\n",
    ));
    assert!(
        !copied.contains("map_move_or"),
        "a value that reads the map keeps its copy: {copied}"
    );
}

#[test]
fn a_list_of_lists_element_is_lent_to_a_method_on_it() {
    let out = cpp(concat!(
        "let w: 1..=10 = read()\n",
        "var ends: [0..w]List<Int> = [0..w] _ => []\n",
        "for i in 0..20: ends[i % w].push(i)\n",
        "var s = 0\n",
        "for q in 0..20:\n",
        "    s += ends[q % w].partition_point(x => x < q)\n",
        "print(s)\n",
    ));
    assert!(
        out.contains("-> decltype(auto) {") && out.contains("return (t"),
        "the element must reach partition_point as a reference: {out}"
    );
}

#[test]
fn let_of_an_element_lends_it_unless_the_list_changes() {
    let src = |after: &str| {
        format!(
            "fn f(n: Int) -> Int:\n    var rows: List<List<Int>> = []\n    for i in 0..n: rows.push([i, i + 1])\n    var s = 0\n    for i in 0..n:\n        let row = rows[(i * 7) % n]\n        for x in row: s += x\n{after}    s\nprint(f(5))\n"
        )
    };
    let lent = cpp(&src(""));
    assert!(
        lent.contains("const std::vector<ll>& row = "),
        "R05.3.5: `let row = rows[i]` lends: {lent}"
    );
    let changed = cpp(&src("        rows[0] = []\n"));
    assert!(
        changed.contains("std::vector<ll> row = "),
        "a list changed later in the loop keeps the copy: {changed}"
    );
}

#[test]
fn a_named_closure_in_a_fn_borrows_a_list_parameter() {
    let out = cpp(concat!(
        "fn count(xs: List<Int>, q: Int) -> Int:\n",
        "    let f = i => xs[i] * 2\n",
        "    var s = 0\n",
        "    for i in 0..q: s += f(i % xs.len)\n",
        "    s\n",
        "print(count([1, 2, 3], 5))\n",
    ));
    assert!(
        out.contains("[&xs_2 = xs]"),
        "the lambda must not copy `xs`: {out}"
    );
    let kept = cpp(concat!(
        "fn make(xs: List<Int>) -> (Int -> Int):\n",
        "    let f = i => xs[i] * 2\n",
        "    f\n",
        "let g = make([1, 2, 3])\n",
        "print(g(1))\n",
    ));
    assert!(
        !kept.contains("[&xs"),
        "a closure that outlives its fn keeps its own copy: {kept}"
    );
}

#[test]
fn walking_a_grouped_map_names_each_entry_without_copying() {
    let out = cpp(concat!(
        "let a = [0..20] i => i\n",
        "let g = a.group(by: x => x % 10)\n",
        "var s = 0\n",
        "for (k, v) in g: s += k * v.len\n",
        "print(s)\n",
    ));
    assert!(
        out.contains("= std::tie("),
        "each entry is a pair of names: {out}"
    );
    assert!(
        !out.contains("cpl::map_get("),
        "`out.get(k) == None` must not copy the entry: {out}"
    );
}

#[test]
fn explain_names_each_copy_and_why_it_owns_its_value() {
    let source = concat!(
        "let rows = [[1, 2], [3, 4]]\n",
        "fn f(rows: List<List<Int>>) -> Int:\n",
        "    let row = rows[1]\n",
        "    var own = rows[0]\n",
        "    own.push(5)\n",
        "    row.len + own.len\n",
        "fn keep(xs: List<Int>) -> (Int -> Int):\n",
        "    i => xs[i]\n",
        "print(f(rows) + keep(rows[0])(1))\n",
    );
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let report = crate::explain::report("t.cp", source, &program);
    assert!(
        report.contains("[copy] 4:15  rows[0] -> the name is changed later"),
        "{report}"
    );
    assert!(
        report.contains("`xs` captured by value -> the closure may be kept"),
        "{report}"
    );
    assert!(
        !report.contains("[copy] 3:"),
        "a lent `let row` is no copy: {report}"
    );
}
