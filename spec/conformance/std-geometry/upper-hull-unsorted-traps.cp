// expect: R0401 (checked)
use std.geometry.{upper_hull}
print(upper_hull([Pt(2, 0), Pt(0, 0)]).len) // <- here
