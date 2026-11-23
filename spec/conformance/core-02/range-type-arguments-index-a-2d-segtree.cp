// expect: ok, output "9\n7\n15"
// Each range argument of `SegTree2D` becomes its own anonymous domain, and a
// value parameter beside them stays a type (decision 255).
use std.segtree.{SegTree2D}
var g = SegTree2D<0..3, 0..3, Int>(+, fill: 1)
print(g.all)
g.set(1, 2, 7)
print(g.get(1, 2))
print(g.all)
