// expect: ok, output "YES NO YES NO YES\nYES\n1 0 0 1 -1 0\n0 1 3\n0 2 3\n1 3 8 12"
use std.geometry.{Pt, Vec, Circle, HalfPlane, Rect, Pt3, Vec3, Plane3, Location, ByAngleFrom, angle_less, arg, lower_hull_idx, upper_hull_idx, pick_interior, pick_boundary, turn_eps, angle_between, invert, incenter, close, line_plane_intersect, sphere_dist}
print(angle_less(Pt(1, 0), Pt(0, 1)), angle_less(Pt(0, -1), Pt(-1, 0)), angle_less(Pt(0, 0), Pt(1, 0)), angle_less(Pt(2, 0), Pt(1, 0)), angle_less(Pt(-1, 0), Pt(1, 0), from: Pt(0, 1)))
print(ByAngleFrom(Pt(0, 1), Pt(-1, 0)) < ByAngleFrom(Pt(0, 1), Pt(1, 0)))
var pts = [Pt(0, 1), Pt(-1, 0), Pt(1, 0)]
sort(mut pts, by: arg)
print(pts)
domain D = 0..4
let ps: [D]Pt = [Pt(0, 0), Pt(1, -1), Pt(1, 1), Pt(2, 0)]
print(lower_hull_idx(ps))
print(upper_hull_idx(ps))
print(pick_interior([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]), pick_interior([Pt(0, 0), Pt(4, 0), Pt(0, 4)]), pick_boundary([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]), pick_boundary([Pt(0, 0), Pt(4, 0), Pt(0, 4)]))
