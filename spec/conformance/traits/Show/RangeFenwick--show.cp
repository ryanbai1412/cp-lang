// expect: ok, output "5 6 3"
use std.fenwick.{RangeFenwick}
var f = RangeFenwick([1, 2, 3])
f.add(0..2, 4)
print(f)
