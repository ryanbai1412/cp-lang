// expect: ok, output "1 2, 3 0\n1 2: 7, 3 0: 0"
use std.fenwick.{SparseFenwick2D}
var f = SparseFenwick2D<Int>()
f.add_point(1, 2)
f.add_point(3, 0)
print(f)
f.build()
f.add(1, 2, 7)
print(f)
