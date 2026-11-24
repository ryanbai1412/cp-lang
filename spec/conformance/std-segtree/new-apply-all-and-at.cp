// expect: ok, output "9 5 0\n16 7"
// rule: std.segtree.SegTree.apply_all
use std.segtree.{SegTree}
use std.algebra.{add}

var s = SegTree(+, [1, 2, 3], act: add)
s.apply_all(1)
print(s.all, SegTree<0..4, Int>(max, fill: 5).fold(1..3), SegTree<0..4, Int>(+).all)
s.at(0) += 5
s.apply(1..3, 1)
print(s.all, s.fold(0..1))
