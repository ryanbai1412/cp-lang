// expect: ok, output "YES YES\nYES\nYES\nYES YES"
use std.slope.{Concave, Convex}
var f = Concave.flat(0..4)
print(yes(f.lo == 0), yes(f.hi == 3))
f.add_quadratic(c => -c * c + 4 * c)
print(yes(f.at(2) == 4))
print(yes(f.max() == 4))
var g = Convex.point(1)
g.min_convolve(Convex.flat(0..3))
print(yes(g.lo == 1 and g.hi == 3), yes(g.at(2) == 0 and g.min() == 0))
