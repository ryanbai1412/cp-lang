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
