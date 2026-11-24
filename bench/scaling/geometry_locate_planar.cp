// n: 1024
// The planar subdivisions of std.geometry: Voronoi, Delaunay, the Manhattan MST, and the faces of
// a grid graph with a point located in each. Points are scattered by multiplying their index by
// large primes.
use std.geometry.*
let n: Int = read()
let pts = [Pt(i * 7919 % 100003, i * 104729 % 100019) for i in 0..n]
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
var acc = 0
for r in 0..4:
    acc += point_location_planar(gp, gedges, queries).len // timed: geometry.point_location_planar
print(acc)
