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
for r in 0..64:
    if convex_polygons_overlap(hull, other): acc += 1 // timed: geometry.convex_polygons_overlap
for r in 0..128:
    acc += minkowski_sum(hull, other).len // timed: geometry.minkowski_sum
    acc += minkowski_lower(half, other_half).len // timed: geometry.minkowski_lower
for r in 0..256:
    acc += minkowski_upper(half, other_half).len // timed: geometry.minkowski_upper
for i in 0..32 * n:
    let q = Pt(i % n, (i * 7919) % top)
    if locate_convex(hull, q) == Location.Inside: acc += 1 // timed: geometry.locate_convex
    acc += hull_extreme(hull, Pt(i % 7 - 3, i % 5 - 1)) // timed: geometry.hull_extreme
    acc += hull_tangents(hull, Pt(-1 - i % 5, (i * 7919) % top)).left // timed: geometry.hull_tangents
    if let (a, b) = hull_line_intersect(hull, Pt(-1, i % top), Pt(n, (i * 31) % top)): acc += a + b // timed: geometry.hull_line_intersect
print(acc, yes(f > 0.0))
