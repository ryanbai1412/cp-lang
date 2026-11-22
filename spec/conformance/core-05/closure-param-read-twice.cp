// expect: ok, output "9 -3"
// rule: 05.10.4
use std.slope.{Convex, Concave}
var f = Convex.flat(0..=3)
f.add_quadratic(x => x * x)
var g = Concave.flat(0..=3)
g.add_quadratic(x => -x * x + 1)
print(f.at(3), g.at(2))
