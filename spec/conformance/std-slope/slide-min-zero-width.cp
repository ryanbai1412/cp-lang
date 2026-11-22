// expect: ok, output "0 0 1"
use std.slope.{Convex}
var f = Convex.point(0)
f.add_const(1)
f.slide_min(0, 0)
print(f.lo, f.hi, f.at(0))
