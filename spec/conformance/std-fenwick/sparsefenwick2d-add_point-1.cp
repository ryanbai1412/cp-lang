// expect: ok, output "YES"
use std.fenwick.{SparseFenwick2D}
var f = SparseFenwick2D<Int>()
f.add_point(5, 7)
f.add_point(5, 7)
f.build()
f.add(5, 7, 3)
print(f.fold(0..10, 0..10) == 3)
