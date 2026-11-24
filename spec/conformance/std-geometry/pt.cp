// expect: ok, output "25 YES YES -2 1 2 -1 YES YES\n2 -3 0 1 0 0 YES\n11 0 1 -2 0 -2 25 YES 7 4\nYES"
use std.geometry.{Vec, Circle, HalfPlane, Rect, Pt3, Vec3, Plane3, ByAngleFrom, angle_less, arg, lower_hull_idx, upper_hull_idx, pick_interior, pick_boundary, turn_eps, angle_between, invert, incenter, close, line_plane_intersect, sphere_dist}
print(Pt(3, 4).norm2, yes(Pt(3_000_000_000, 4_000_000_000).norm2_128 == I128(25) * I128(1_000_000_000_000_000_000) and Pt(3, 4).norm2_128 == I128(25)), yes(Pt(3, 4).len == 5.0), Pt(1, 2).perp, Pt(1, 2).perp_cw, yes(Pt(1, 0).angle == 0.0 and Pt(0, -1).angle < 0.0 and Pt(0, 0).angle == 0.0), yes(Pt(-1, 0).angle > 3.14))
print(Pt(4, -6).unit_dir(), Pt(0, 5).unit_dir(), Pt(0, 0).unit_dir(), yes(Pt(1, 2).to_vec == Vec(1.0, 2.0)))
print(dot(Pt(1, 2), Pt(3, 4)), dot(Pt(1, 0), Pt(0, 1)), cross(Pt(1, 0), Pt(0, 1)), cross(Pt(1, 2), Pt(3, 4)), cross(Pt(1, 2), Pt(2, 4)), Pt(1, 2).cross128(Pt(3, 4)), Pt(0, 0).dist2(Pt(3, 4)), yes(Pt(0, 0).dist(Pt(3, 4)) == 5.0 and Pt(1, 2).dot128(Pt(3, 4)) == I128(11) and Pt(0, 0).dist2_128(Pt(3, 4)) == I128(25)), Pt(1, 1).manhattan(Pt(4, -3)), Pt(1, 1).chebyshev(Pt(4, -3)))
print(yes(Pt(1, 1).dist2(Pt(1, 1)) == 0))
