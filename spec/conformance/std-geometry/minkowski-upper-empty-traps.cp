// expect: R0401 at 3
use std.geometry.{minkowski_upper}
let a: List<Pt> = []
print(minkowski_upper(a, [Pt(0, 0)]).len)
