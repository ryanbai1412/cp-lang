// expect: ok, output "2\n4"
use std.geometry.{convex_hull_idx}
print(convex_hull_idx([Pt(0, 0), Pt(1, 1), Pt(2, 2)]).len)
print(convex_hull_idx([Pt(0, 0), Pt(1, 0), Pt(1, 1), Pt(0, 1), Pt(0, 0)]).len)
