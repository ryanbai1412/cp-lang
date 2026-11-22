// expect: ok, output "1\n6"
// rule: 04.1.1
use std.segtree.{SegTree}
print(SegTree(min, [4, 1, 9]).fold(0..3))
print(SegTree(+, [1, 2, 3]).fold(0..3))
