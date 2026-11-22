// expect: ok, output "YES"
use std.fenwick.{SparseFenwick2D}
var f = SparseFenwick2D<Int>()
f.add_point(1, 2)
f.build()
print(yes(f.fold(1..2, 2..3) == 0))
