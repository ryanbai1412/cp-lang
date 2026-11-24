// n: 1024
// The point-set, sweep, planar and 3D algorithms of std.geometry. Points are scattered by
// multiplying their index by large primes; the 3D points lie on a paraboloid, so all are corners.
use std.geometry.*
let n: Int = read()
let pts = [Pt(i * 7919 % 100003, i * 104729 % 100019) for i in 0..n]
let vs = [Vec(F64(p.x), F64(p.y)) for p in pts]
let segs = [(a: Pt(i * 37 % 1000, i), b: Pt(i * 37 % 1000 + 500, i)) for i in 0..n]
let rects = [Rect(i * 7919 % 100003..i * 7919 % 100003 + 5000, i * 104729 % 100019..i * 104729 % 100019 + 5000) for i in 0..n]
let circles = [Circle(Vec(F64(i * 37 % 1000), F64(i * 91 % 1000)), 50.0) for i in 0..n / 8]
let w = 32
var grid: List<Pt> = []
var edges: List<(Int, Int)> = []
var queries: List<Pt> = []
for i in 0..n:
    let (r, c) = (i / w, i % w)
    grid.push(Pt(2 * c, 2 * r))
    if c + 1 < w: edges.push((i, i + 1))
    if i + w < n: edges.push((i, i + w))
    queries.push(Pt(2 * c + 1, 2 * r + 1))
domain G = 0..n
let gp = [G] g => grid[g.ord]
let gedges = [(G(u), G(v)) for (u, v) in edges]
let pts3 = [Pt3(i * 7919 % 10007, i * 104729 % 10009, (i * 7919 % 10007) * (i * 7919 % 10007) + (i * 104729 % 10009) * (i * 104729 % 10009)) for i in 0..n / 4]
var acc = 0
var f = 0.0
for r in 0..2:
    acc += voronoi_cells(pts, 2e5)[0].len // timed: geometry.voronoi_cells
for r in 0..4:
    acc += manhattan_mst(pts).len // timed: geometry.manhattan_mst
    acc += delaunay(pts).len // timed: geometry.delaunay
    acc += planar_faces(gp, gedges).len // timed: geometry.planar_faces
    acc += point_location_planar(gp, gedges, queries).len // timed: geometry.point_location_planar
    acc += convex_hull_3d(pts3).len // timed: geometry.convex_hull_3d
    acc += convex_hull_3d_idx(pts3).len // timed: geometry.convex_hull_3d_idx
for r in 0..8:
    acc += (any_segments_meet(segs) ?? (0, 0)).0 // timed: geometry.any_segments_meet
    acc += (any_segments_meet_idx(segs) ?? (0, 0)).0 // timed: geometry.any_segments_meet_idx
    acc += rects_union_area(rects) % 7 // timed: geometry.rects_union_area
for r in 0..32:
    acc += closest_pair2(pts) % 7 // timed: geometry.closest_pair2
    acc += closest_pair(pts).dist2 % 7 // timed: geometry.closest_pair
for r in 0..64:
    f += circles_union_area(circles) // timed: geometry.circles_union_area
for r in 0..128:
    f += min_enclosing_circle(vs).r // timed: geometry.min_enclosing_circle
let faces = convex_hull_3d(pts3)
for r in 0..2048:
    acc += Int(polyhedron_volume6(pts3, faces) % I128(7)) // timed: geometry.polyhedron_volume6
print(acc, yes(f > 0.0))
