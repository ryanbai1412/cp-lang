// expect: R0401 at 2 (checked)
use std.geometry.{polygon_triangulate}
print(polygon_triangulate([Pt(0, 0), Pt(0, 1), Pt(1, 0)]).len)
