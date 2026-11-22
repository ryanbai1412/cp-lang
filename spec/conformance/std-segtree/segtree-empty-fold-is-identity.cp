// expect: ok, output "0 0"
// rule: std.segtree.SegTree.fold
use std.segtree.{SegTree}
domain D = 0..3
domain E = 0..0
let s = SegTree<D, Int>(+, fill: 2)
print(s.fold(D(1)..D(1)), SegTree<E, Int>(+).all)
