// expect: ok, output "0 0"
use std.slope.{Convex}
var f = Convex.flat(0..=4)
f.suffix_min()
print(f.at(0), f.at(4))
