// expect: ok, output "2 7 5 8"
use std.segtree.{SegTree}
var s = SegTree(max, [2, 7, 1, 8])
s.set(2, 5)
print(s)
