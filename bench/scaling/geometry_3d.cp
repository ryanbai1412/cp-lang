// n: 1024
// The 3D hull of std.geometry and the volume of what it bounds. The points lie on a paraboloid,
// so all are corners.
use std.geometry.*
let n: Int = read()
let pts3 = [Pt3(i * 7919 % 10007, i * 104729 % 10009, (i * 7919 % 10007) * (i * 7919 % 10007) + (i * 104729 % 10009) * (i * 104729 % 10009)) for i in 0..n / 4]
var acc = 0
for r in 0..4:
    acc += convex_hull_3d(pts3).len // timed: geometry.convex_hull_3d
    acc += convex_hull_3d_idx(pts3).len // timed: geometry.convex_hull_3d_idx
let faces = convex_hull_3d(pts3)
for r in 0..2048:
    acc += Int(polyhedron_volume6(pts3, faces) % I128(7)) // timed: geometry.polyhedron_volume6
print(acc)
