// n: 2048
// polygon_triangulate on a convex polygon of 2n corners: the points of two parabolas, the lower
// one rising and the upper one falling, so every point is a corner.
use std.geometry.*
let n: Int = read()
var acc = 0
let top = 2 * n * n
var hull: List<Pt> = []
for i in 0..n: hull.push(Pt(i, i * i))
for i in 0..n: hull.push(Pt(n - 1 - i, top - (n - 1 - i) * (n - 1 - i)))
for r in 0..2:
    acc += polygon_triangulate(hull).len // timed: geometry.polygon_triangulate
print(acc)
