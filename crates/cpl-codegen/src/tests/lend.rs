//! Values the C++ reads in place instead of copying.

use super::body;

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
