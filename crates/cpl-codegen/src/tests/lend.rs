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
    let printed = cpp(&src("    print(o!)\n"));
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
        "    let l: List<Int> = m.get(0) ?? []\n",
        "    s += l.bsearch(x => x >= hi)\n",
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
        "    s += ends[q % w].bsearch(x => x >= q)\n",
        "print(s)\n",
    ));
    assert!(
        out.contains("-> decltype(auto) {") && out.contains("return (t"),
        "the element must reach bsearch as a reference: {out}"
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
        changed.contains("const std::vector<ll>& row = "),
        "a store after row's last use leaves it lent: {changed}"
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
    let mut db = cpl_check::Session::new();
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

#[test]
fn a_let_of_a_list_set_again_before_any_read_takes_it() {
    let taken = cpp(concat!(
        "struct S:\n",
        "    var ks: List<Int>\n",
        "    var vs: List<Int>\n",
        "fn grow(mut s: S):\n",
        "    let old_k = s.ks\n",
        "    let old_v = s.vs\n",
        "    s.ks = List.filled(4, 0)\n",
        "    s.vs = List.filled(4, 0)\n",
        "    for i in 0..old_k.len: s.ks[i] = old_k[i] + old_v[i]\n",
        "var s = S(ks: [1, 2], vs: [3, 4])\n",
        "grow(mut s)\n",
        "print(s.ks)\n",
    ));
    assert!(taken.contains("old_k = std::move("), "{taken}");
    assert!(taken.contains("old_v = std::move("), "{taken}");
    let read_first = cpp(concat!(
        "struct S:\n",
        "    var ks: List<Int>\n",
        "fn grow(mut s: S):\n",
        "    let old_k = s.ks\n",
        "    print(s.ks.len)\n",
        "    s.ks = List.filled(4, 0)\n",
        "    print(old_k)\n",
        "var s = S(ks: [1, 2])\n",
        "grow(mut s)\n",
    ));
    assert!(!read_first.contains("old_k = std::move("), "{read_first}");
}

#[test]
fn a_read_lends_when_its_source_changes_only_after_the_last_read() {
    let lent = body(concat!(
        "var rows = [[1, 2], [3]]\n",
        "var k = 1\n",
        "while k < 3:\n",
        "    let r = rows[k - 1]\n",
        "    var s = 0\n",
        "    for x in r:\n",
        "        let y = r[0]\n",
        "        s += x + y\n",
        "    rows.push([s])\n",
        "    k += 1\n",
        "print(rows.len)\n",
    ));
    assert!(
        lent.contains("const std::vector<ll>& r = "),
        "the push comes after r's last read: {lent}"
    );
}

#[test]
fn a_local_read_for_the_last_time_moves() {
    let src = |take: &str, tail: &str| {
        format!(
            "fn f(n: Int) -> Int:\n    var lo: List<Int> = []\n    for i in 0..n: lo.push(i)\n    var res = {take}\n    res.push(9)\n    res.len{tail}\nprint(f(3))\n"
        )
    };
    let moved = cpp(&src("lo", ""));
    assert!(moved.contains("std::move(lo)"), "lo is dead after: {moved}");
    let copied = cpp(&src("lo.clone()", " + lo.len"));
    assert!(
        !copied.contains("std::move(lo)"),
        "lo is read after: {copied}"
    );
}

#[test]
fn a_let_in_a_value_block_lends_when_the_value_does_not_read_it() {
    let out = cpp(concat!(
        "domain C = 0..3\n",
        "var b: [C]C = [C] p => p\n",
        "b.swap(C(0), C(2))\n",
        "let at = b.inverse()\n",
        "print(at[C(0)])\n",
    ));
    assert!(
        out.lines()
            .any(|l| l.contains("& t") && l.trim_end().ends_with("= b;")),
        "inverse reads b in place: {out}"
    );
}

#[test]
fn a_top_level_closure_only_called_borrows_a_list_a_loop_walks() {
    let out = cpp("let n = read<Int>()
for v in 0..n:
    let kids = [v, v + 1]
    let need = (r: Int) => kids.sum(of: c => c + r)
    var best = 0
    for c in kids: best += c
    print(need(best))
");
    assert!(
        out.contains("&kids_2 = kids"),
        "`need` is only called and `kids` never changes, so it is borrowed:\n{out}"
    );
}

#[test]
fn a_local_fn_called_by_itself_or_another_local_fn_borrows_its_captures() {
    let head =
        "let xs = read<List<Int>>()\nfn f(k: Int) -> Int:\n    let ys = [x * k for x in xs]\n";
    for body in [
        "    fn walk(i: Int) -> Int:\n        if i >= ys.len: return 0\n        ys[i] + walk(i + 1)\n    walk(0)\n",
        "    fn at(i: Int) -> Int = ys[i]\n    fn g(n: Int) -> Int:\n        var s = 0\n        for i in 0..n: s += at(i)\n        s\n    g(ys.len)\n",
    ] {
        let out = cpp(&format!("{head}{body}print(f(2))\n"));
        assert!(
            out.contains("&ys_2 = ys"),
            "the local fn is only ever called, so `ys` is borrowed:\n{out}"
        );
    }
    let kept = cpp(concat!(
        "let xs = read<List<Int>>()\n",
        "fn f(k: Int) -> Int:\n",
        "    let ys = [x * k for x in xs]\n",
        "    fn at(i: Int) -> Int = ys[i]\n",
        "    var h = (i: Int) => at(i)\n",
        "    if k > 1: h = (i: Int) => i\n",
        "    h(0)\n",
        "print(f(2))\n",
    ));
    assert!(
        !kept.contains("&ys_2 = ys"),
        "a closure stored in a var keeps `at`, so `at` owns `ys`:\n{kept}"
    );
}

#[test]
fn sorted_at_the_last_read_sorts_the_list_itself() {
    let out = cpp(concat!(
        "let es: [0..3](a: Int, w: Int) = read()\n",
        "for (a, w) in es:\n",
        "    print(a)\n",
        "for (a, w) in es.sorted(by: e => -e.w):\n",
        "    print(a, w)\n",
    ));
    assert!(out.contains("std::move(es)"), "{out}");
    assert!(!out.contains("= es;"), "{out}");
}

#[test]
fn sorted_before_another_read_or_inside_a_loop_copies() {
    for tail in [
        "for (a, w) in es.sorted(by: e => -e.w):\n    print(a, es.len)\n",
        "for (a, w) in es.sorted(by: e => -e.w):\n    print(a)\nprint(es.len)\n",
        "for i in 0..2:\n    for (a, w) in es.sorted(by: e => -e.w):\n        print(a)\n",
    ] {
        let out = cpp(&format!("let es: [0..3](a: Int, w: Int) = read()\n{tail}"));
        assert!(!out.contains("std::move(es)"), "{tail}: {out}");
    }
}

#[test]
fn an_entry_a_store_overwrites_is_moved_into_the_fn_that_makes_it() {
    let moved = cpp(concat!(
        "var h = Map<Int, List<Int>>()\n",
        "for v in 0..3:\n",
        "    h.update(v % 2, [], l => l + [v])\n",
        "print(h.len)\n",
    ));
    assert!(
        moved.contains("cpl::map_move_or(self, "),
        "`update` moves the entry into `f`, not a copy: {moved}"
    );
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
fn a_named_closure_borrows_each_capture_nothing_changes() {
    let out = cpp(concat!(
        "let n: Int = read()\n",
        "var door = [0 for i in 0..3]\n",
        "for d in 0..3:\n",
        "    let free = [i for i in 0..n if i != d]\n",
        "    fn opens(k: Int) -> Bool = free[..k].len + door[0] > 2\n",
        "    door[0] = d\n",
        "    print(yes(opens(d)))\n",
    ));
    assert!(
        out.contains("&free"),
        "`free` never changes, so it is borrowed: {out}"
    );
    assert!(
        !out.contains("&door"),
        "`door` changes, so it is copied: {out}"
    );
}

#[test]
fn a_named_closure_borrows_its_unchanged_captures_beside_a_changed_one() {
    let out = cpp(concat!(
        "let n = read::<Int>()\n",
        "fn go() -> Int:\n",
        "    var seen = [0 for _ in 0..n]\n",
        "    let free = [i for i in 0..n]\n",
        "    fn ok(k: Int) -> Bool:\n",
        "        seen[0] += 1\n",
        "        free.len >= k\n",
        "    seen[0] = 1\n",
        "    let k = (0..=n).first_true(k => not ok(k)) ?? 0\n",
        "    k + seen[0] + free.len\n",
        "print(go())\n",
    ));
    assert!(
        out.contains("&free_2 = free"),
        "an unchanged list is borrowed, not copied, though `seen` changes: {out}"
    );
}

#[test]
fn a_closure_borrows_a_list_moved_only_after_its_last_call() {
    let lent = cpp(concat!(
        "let n = read::<Int>()\n",
        "let around: [0..n]List<Int> = read()\n",
        "fn walk(u: 0..n, p: Int) -> Int:\n",
        "    let doors = around[u]\n",
        "    let back = (0..doors.len).find(i => doors[i] == p) ?? 0\n",
        "    [c for c in doors[back..]].len + [c for c in doors].len\n",
        "print(walk(0, read()))\n",
    ));
    assert!(lent.contains("&doors_2 = doors"), "{lent}");
    let kept = cpp(concat!(
        "let n = read::<Int>()\n",
        "let around: [0..n]List<Int> = read()\n",
        "fn walk(u: 0..n, p: Int) -> Int:\n",
        "    let doors = around[u]\n",
        "    let has = (i: Int) => doors[i] == p\n",
        "    let seen = doors\n",
        "    seen.len + (if has(0): 1 else: 0)\n",
        "print(walk(0, read()))\n",
    ));
    assert!(!kept.contains("&doors_2 = doors"), "{kept}");
}

#[test]
fn a_top_level_list_sorted_at_its_last_read_moves_after_a_comprehension() {
    let out = cpp(concat!(
        "let m = read::<Int>()\n",
        "let cords: [m](x: Int, y: Int) = read()\n",
        "let xs = [c.x for c in cords]\n",
        "let tall = cords.sorted(by: c => -c.y)\n",
        "print(xs.len, tall.len)\n",
    ));
    assert!(out.contains("std::move(cords)"), "{out}");
    let viewed = cpp(concat!(
        "let m = read::<Int>()\n",
        "let cords: [m](x: Int, y: Int) = read()\n",
        "let head = cords[..1]\n",
        "let tall = cords.sorted(by: c => -c.y)\n",
        "print(head.len, tall.len)\n",
    ));
    assert!(!viewed.contains("std::move(cords)"), "{viewed}");
}

#[test]
fn a_closure_handed_to_a_fn_that_recurses_with_it_borrows_its_captures() {
    let out = cpp(concat!(
        "fn walk(n: Int, f: Int -> Int) -> Int:\n",
        "    if n == 0: return f(0)\n",
        "    f(n) + walk(n - 1, f)\n",
        "let n: 1..=9 = read()\n",
        "for k in 0..n:\n",
        "    let b = [i * k for i in 0..n]\n",
        "    let f = (i: Int) => b[i % n]\n",
        "    print(walk(n, f), f(1))\n",
    ));
    assert!(out.contains("&b_2 = b"), "{out}");
    assert!(!out.contains("[b_2 = b]"), "{out}");
}

#[test]
fn a_method_value_sort_key_lends_its_receiver() {
    let out = cpp(concat!(
        "struct Ranks(r: List<Int>):\n",
        "    fn pos(self, i: Int) -> Int = r[i]\n",
        "let t = Ranks([read<Int>(), 1, 2])\n",
        "let xs = [0, 1, 2]\n",
        "print(xs.sorted(by: t.pos))\n",
        "print(t.r.len)\n",
    ));
    assert!(
        !out.contains("self = t]") && !out.contains("self = t4]"),
        "a key only called inside the sort must not copy its receiver:\n{out}"
    );
}

#[test]
fn a_local_fn_another_local_fn_only_calls_lends_its_captures() {
    let out = cpp(concat!(
        "fn solve(n: Int):\n",
        "    let t = [read<Int>() for _ in 0..n]\n",
        "    fn get(i: Int) -> Int = t[i]\n",
        "    fn walk(i: Int):\n",
        "        if i < n:\n",
        "            print(get(i))\n",
        "            walk(i + 1)\n",
        "    walk(0)\n",
        "solve(read<Int>())\n",
    ));
    assert!(
        !out.contains("[t_2 = t") && !out.contains(", t_2 = t"),
        "`get` is only called, by `walk` which is only called, so it borrows `t`:\n{out}"
    );
}

#[test]
fn a_local_fn_inside_a_local_fn_borrows_what_it_only_reads() {
    let out = cpp(concat!(
        "fn f(k: Int) -> Int:\n",
        "    fn outer(n: Int) -> Int:\n",
        "        let ys = [i * k for i in 0..n]\n",
        "        fn at(i: Int) -> Int = ys[i]\n",
        "        at(0) + at(n - 1)\n",
        "    outer(3)\n",
        "print(f(read()))\n",
    ));
    assert!(
        !out.contains("[ys_2 = ys") && !out.contains(" ys_2 = ys"),
        "a nested local fn that is only called must not copy its capture:\n{out}"
    );
}

#[test]
fn a_match_binding_of_a_read_string_lends_the_scrutinee() {
    let out = cpp(concat!(
        "let q: 1..=10 = read()\n",
        "var n = 0\n",
        "for _ in 0..q:\n",
        "    match read<Str>():\n",
        "        \"ADD\" => n += 1\n",
        "        kind =>\n",
        "            let l: 1..=10 = read()\n",
        "            if kind == \"SUB\": n -= l else: n += l\n",
        "print(n)\n",
    ));
    assert!(
        !out.contains("std::string kind ="),
        "a name bound to a scrutinee it only reads must not copy it:\n{out}"
    );
}
