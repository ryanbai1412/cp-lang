// expect: ok, output "3\n11"
// rule: 06.1.8
use std.segtree.{SegTree}
let a = [5, 3, 8, 1]
print(SegTree(min, a).fold(1..3))
print(SegTree(+, a).fold(1..3))
