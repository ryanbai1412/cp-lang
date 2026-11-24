// expect: R0401 (checked)
use std.geometry.{hull_extreme}
let empty: List<Pt> = []
print(hull_extreme(empty, Pt(1, 0))) // <- here
