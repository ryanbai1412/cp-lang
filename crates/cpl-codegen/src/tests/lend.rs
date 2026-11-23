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
        "var c = 0\n",
        "for i in 0..10:\n",
        "    if h.less(i, 5): c += 1\n",
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
