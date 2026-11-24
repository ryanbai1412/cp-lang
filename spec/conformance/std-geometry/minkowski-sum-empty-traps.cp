// expect: R0401 (checked)
use std.geometry.{minkowski_sum}
let e: List<Pt> = []
print(minkowski_sum(e, [Pt(0, 0)]).len) // <- here
