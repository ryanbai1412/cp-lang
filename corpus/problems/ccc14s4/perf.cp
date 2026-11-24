// Source: CCC 2014 Senior S4 (Tinted Glass Window), https://dmoj.ca/problem/ccc14s4
// Tuned: port.cp swept along x, keeping one column of y differences that each pane's edge events
//   update, instead of the whole difference grid: O(n) memory, as the original.
// limits: 1000 ms, 256 MB
// Problem: n <= 1000 axis-parallel rectangles, corners x_l y_t x_r y_b in 0..=K with K <= 1e9,
//   each adding tint t_i; overlaps add up. Print the total area whose tint is at least
//   T <= 1e9 (the answer can exceed 2^32 but fits 64 bits).
// Solution: the 2n pane edges cut each axis into at most 2n - 1 strips, and the tint is constant
//   on every cell of that grid. Compress both axes with std `compress`, mark each pane's four
//   corners in a difference grid, take running sums along both axes and add the area of every cell dark enough: O(n^2)
//   cells, no data structure (the original sweeps x over a sqrt-decomposed y axis).
// original: corpus/submissions/5672281.cpp
// Bounds: n: 1..=1000; need, panes.tint: 1..=1e9;
//   panes.x0, panes.y0, panes.x1, panes.y1: 0..=1e9
let n: 1..=1000 = read()
let need: 1..=1e9 = read()
let panes: [n](x0: 0..=1e9, y0: 0..=1e9, x1: 0..=1e9, y1: 0..=1e9, tint: 1..=1e9) = read()

let (domain X, xs) = compress([p.x0 for p in panes] + [p.x1 for p in panes])
let (domain Y, ys) = compress([p.y0 for p in panes] + [p.y1 for p in panes])

var edges: [X]List<(y: Y, by: Int)> = []
for p in panes:
    let (a, b) = (xs.index_of(p.x0), xs.index_of(p.x1))
    let (c, e) = (ys.index_of(p.y0), ys.index_of(p.y1))
    edges[a].push((y: c, by: p.tint))
    edges[a].push((y: e, by: -p.tint))
    edges[b].push((y: c, by: -p.tint))
    edges[b].push((y: e, by: p.tint))

var column: [Y]Int = 0  // the y differences of the strip right of the current x
var area = 0
for (i, i2) in X.all.adjacent:
    for (y, by) in edges[i]: column[y] += by
    var tint = 0
    for (j, j2) in Y.all.adjacent:
        tint += column[j]
        if tint >= need: area += (xs.get(i2) - xs.get(i)) * (ys.get(j2) - ys.get(j))
print(area)
