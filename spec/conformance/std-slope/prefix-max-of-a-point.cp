// expect: ok, output "3"
use std.slope.{Concave}
var f = Concave.point(0)
f.add_const(3)
f.prefix_max()
print(f.at(0))
