// expect: R0401 (checked)
use std.slope.{Concave}
var f = Concave.point(0)
f.slide_max(2, -1) // <- here
print(f.hi)
