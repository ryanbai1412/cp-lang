// expect: ok, output "YES"
use std.fenwick.{SparseFenwick2D}
var f = SparseFenwick2D<Int>()
f.add_point(1, 2)
f.build()
f.add(1, 2, 5)
print(f.fold(2..2, 0..10) == 0)
