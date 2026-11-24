// expect: ok, output "5 2 3"
use std.fenwick.{Fenwick}
var f = Fenwick([1, 2, 3])
f.add(0, 4)
print(f)
