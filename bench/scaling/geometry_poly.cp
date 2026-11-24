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
for r in 0..8:
    acc += convex_polygon_intersect(vs, ovs).len // timed: geometry.convex_polygon_intersect
for r in 0..16:
    acc += polygon_kernel(vs).len // timed: geometry.polygon_kernel
    acc += half_plane_intersect([HalfPlane.through(vs[i], vs[(i + 1) % vs.len]) for i in 0..vs.len]).len // timed: geometry.half_plane_intersect
for r in 0..64:
    f += circle_polygon_area(Circle(Vec(F64(n) / 2.0, mid), F64(n) / 3.0), vs) // timed: geometry.circle_polygon_area
for r in 0..128:
    acc += pick_boundary(hull) % 7 // timed: geometry.pick_boundary
    acc += pick_interior(hull) % 7 // timed: geometry.pick_interior
for r in 0..256:
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
print(acc, yes(f > 0.0))
