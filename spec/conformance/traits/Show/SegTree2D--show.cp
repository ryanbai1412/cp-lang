// expect: ok, output "0 0 4 / 0 6 0"
use std.segtree.{SegTree2D}
var g = SegTree2D<0..2, 0..3, Int>(+, fill: 0)
g.set(0, 2, 4)
g.set(1, 1, 6)
print(g)
