// expect: R0401 at 2 (checked)
use std.geometry.{hull_tangents}
print(hull_tangents([Pt(0, 0), Pt(2, 0)], Pt(4, 1)).left)
