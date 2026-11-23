// expect: ok, output "0 1"
use std.geometry.{upper_hull}
let empty: List<Pt> = []
print(upper_hull(empty).len, upper_hull([Pt(3, 3)]).len)
