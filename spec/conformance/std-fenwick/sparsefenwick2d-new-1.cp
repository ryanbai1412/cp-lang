// expect: ok, output "YES"
use std.fenwick.{SparseFenwick2D}
var f = SparseFenwick2D<Int>()
f.build()
print(yes(f.fold(0..10, 0..10) == 0))
