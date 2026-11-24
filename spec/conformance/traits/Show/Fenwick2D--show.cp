// expect: ok, output "0 0 0\n0 0 5"
use std.fenwick.{Fenwick2D}
domain R = 0..2
domain C = 0..3
var f = Fenwick2D<R, C, Int>()
f.add(R(1), C(2), 5)
print(f)
