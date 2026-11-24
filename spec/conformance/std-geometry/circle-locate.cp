// expect: ok, output "YES YES YES\nYES"
use std.geometry.{Vec, Circle, HalfPlane, Rect, Pt3, Vec3, Plane3, ByAngleFrom, arg, lower_hull_idx, upper_hull_idx, pick_interior, pick_boundary, turn_eps, angle_between, invert, incenter, line_plane_intersect, sphere_dist}
let c = Circle(Vec(0.0, 0.0), 2.0)
print(yes(c.locate(Vec(1.0, 0.0)) == Location.Inside), yes(c.locate(Vec(2.0, 0.0)) == Location.Boundary), yes(c.locate(Vec(3.0, 0.0)) == Location.Outside))
let i = incenter(Vec(0.0, 0.0), Vec(4.0, 0.0), Vec(0.0, 3.0))!
print(yes(close(i.c.x, 1.0) and close(i.c.y, 1.0) and close(i.r, 1.0) and incenter(Vec(0.0, 0.0), Vec(1.0, 1.0), Vec(2.0, 2.0)) == None))
