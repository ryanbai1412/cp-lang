// expect: ok, output "1 2 2"
use std.fenwick.{XorFenwick}
var f = XorFenwick([1, 2, 3])
f.add(2, 1)
print(f)
