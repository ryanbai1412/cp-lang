// n: 2048
// The polygon and hull fns of std.geometry on a convex polygon of 2n corners: the points of two
// parabolas, the lower one rising and the upper one falling, so every point is a corner.
use std.geometry.*
let n: Int = read()
var acc = 0
var f = 0.0
let top = 2 * n * n
var sorted_pts: List<Pt> = []
for i in 0..n:
    sorted_pts.push(Pt(i, i * i))
    sorted_pts.push(Pt(i, top - i * i))
var scattered: List<Pt> = []
for k in 0..2 * n: scattered.push(sorted_pts[k * 40503 % (2 * n)])
var hull: List<Pt> = []
for i in 0..n: hull.push(Pt(i, i * i))
for i in 0..n: hull.push(Pt(n - 1 - i, top - (n - 1 - i) * (n - 1 - i)))
let vs = [Vec(F64(p.x), F64(p.y)) for p in hull]
let other = [Pt(p.x + 3, p.y + 5) for p in hull]
let ovs = [Vec(v.x + 0.5, v.y + 0.25) for v in vs]
let half = [Pt(p.x, p.y) for p in hull[0..n]]
let other_half = [Pt(p.x + 1, p.y + 2) for p in hull[0..n]]
let mid = F64(top) / 2.0
for r in 0..2:
    acc += polygon_triangulate(hull).len // timed: geometry.polygon_triangulate
for r in 0..8:
    acc += convex_polygon_intersect(vs, ovs).len // timed: geometry.convex_polygon_intersect
for r in 0..16:
    acc += farthest_pair(scattered).dist2 % 7 // timed: geometry.farthest_pair
    acc += polygon_kernel(vs).len // timed: geometry.polygon_kernel
    acc += half_plane_intersect([HalfPlane.through(vs[i], vs[(i + 1) % vs.len]) for i in 0..vs.len]).len // timed: geometry.half_plane_intersect
for r in 0..32:
    acc += convex_hull(scattered).len // timed: geometry.convex_hull
    acc += convex_hull_idx(scattered).len // timed: geometry.convex_hull_idx
for r in 0..64:
    if convex_polygons_overlap(hull, other): acc += 1 // timed: geometry.convex_polygons_overlap
    acc += diameter2(hull) % 7 // timed: geometry.diameter2
    f += min_bounding_box(hull).area // timed: geometry.min_bounding_box
    f += hull_width(hull) // timed: geometry.hull_width
    f += circle_polygon_area(Circle(Vec(F64(n) / 2.0, mid), F64(n) / 3.0), vs) // timed: geometry.circle_polygon_area
for r in 0..128:
    acc += lower_hull(sorted_pts).len // timed: geometry.lower_hull
    acc += upper_hull(sorted_pts).len // timed: geometry.upper_hull
    acc += lower_hull_idx(sorted_pts).len // timed: geometry.lower_hull_idx
    acc += upper_hull_idx(sorted_pts).len // timed: geometry.upper_hull_idx
    acc += pick_boundary(hull) % 7 // timed: geometry.pick_boundary
    acc += pick_interior(hull) % 7 // timed: geometry.pick_interior
    acc += minkowski_sum(hull, other).len // timed: geometry.minkowski_sum
    acc += minkowski_lower(half, other_half).len // timed: geometry.minkowski_lower
for r in 0..256:
    acc += minkowski_upper(half, other_half).len // timed: geometry.minkowski_upper
    acc += polygon_cut(vs, Vec(0.0, mid + F64(r)), Vec(1.0, mid)).len // timed: geometry.polygon_cut
for r in 0..512:
    f += polygon_centroid(vs).x // timed: geometry.polygon_centroid
for r in 0..1024:
    acc += Int(area2_128(hull) % 7) // timed: geometry.area2_128
for r in 0..2048:
    acc += area2(hull) % 7 // timed: geometry.area2
    f += polygon_area(vs) // timed: geometry.polygon_area
    if locate(hull, Pt(r + 1, top / 2)) == Location.Inside: acc += 1 // timed: geometry.locate
for i in 0..32 * n:
    let q = Pt(i % n, (i * 7919) % top)
    if locate_convex(hull, q) == Location.Inside: acc += 1 // timed: geometry.locate_convex
    acc += hull_extreme(hull, Pt(i % 7 - 3, i % 5 - 1)) // timed: geometry.hull_extreme
    acc += hull_tangents(hull, Pt(-1 - i % 5, (i * 7919) % top)).left // timed: geometry.hull_tangents
    if let (a, b) = hull_line_intersect(hull, Pt(-1, i % top), Pt(n, (i * 31) % top)): acc += a + b // timed: geometry.hull_line_intersect
print(acc, yes(f > 0.0))
