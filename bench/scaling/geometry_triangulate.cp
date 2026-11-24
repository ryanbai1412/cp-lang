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
for i in 0..32 * n:
    let q = Pt(i % n, (i * 7919) % top)
print(acc, yes(f > 0.0))
