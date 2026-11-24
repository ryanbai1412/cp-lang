// expect: ok, output "1 7 8"
use std.fenwick.{DualFenwick}
var d = DualFenwick([1, 2, 3])
d.add(1..3, 5)
print(d)
