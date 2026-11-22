// expect: ok, output "YES"
use std.fenwick.{SparseFenwick2D}
var f = SparseFenwick2D<Int>()
f.add_point(1, 2)
f.add_point(3, 4)
f.build()
f.add(1, 2, 5)
f.add(3, 4, 7)
print(yes(f.fold(0..2, 0..10) == 5))
