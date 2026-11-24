// expect: ok, output "0"
use std.algebra.{Action}
let neg = Action<Int, Bool>(id: false, compose: (x, y) => x != y, apply: (t, v, n) => if t: -v else: v)
var g = SegTree(+, [1, 2, 3], act: neg)
g.apply(0..2, true)
print(g.all)
