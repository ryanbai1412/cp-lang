// expect: ok, output "0 0 4"
use std.slope.{Concave}
var f = Concave.point(0)
f.add_const(4)
f.max_convolve(Concave.flat(0..=0))
print(f.lo, f.hi, f.at(0))
