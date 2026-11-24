// Source: CCC 2014 Senior S4 (Tinted Glass Window), https://dmoj.ca/problem/ccc14s4
// limits: 1000 ms, 256 MB
// Problem: n <= 1000 axis-parallel rectangles, corners x_l y_t x_r y_b in 0..=K with K <= 1e9,
//   each adding tint t_i; overlaps add up. Print the total area whose tint is at least
//   T <= 1e9 (the answer can exceed 2^32 but fits 64 bits).
// Solution: the 2n pane edges cut each axis into at most 2n - 1 strips, and the tint is constant
//   on every cell of that grid. Compress both axes with std `compress`, mark each pane's four
//   corners in a difference grid, take running sums along both axes and add the area of every cell dark enough: O(n^2)
//   cells, no data structure (the original sweeps x over a sqrt-decomposed y axis).
// original: corpus/submissions/5672281.cpp
let n: 1..=1000 = read()
let need: 1..=1e9 = read()
let panes: [n](x0: 0..=1e9, y0: 0..=1e9, x1: 0..=1e9, y1: 0..=1e9, tint: 1..=1e9) = read()

let (domain X, xs) = compress([p.x0 for p in panes] + [p.x1 for p in panes])
let (domain Y, ys) = compress([p.y0 for p in panes] + [p.y1 for p in panes])

// tint[i, j] is the tint of the cell from xs[i] to the next cut by ys[j] to the next: mark
// each pane's corners, then take running sums along both axes.
var tint: [X, Y]Int = 0
for p in panes:
    let (a, b) = (xs.index_of(p.x0), xs.index_of(p.x1))
    let (c, e) = (ys.index_of(p.y0), ys.index_of(p.y1))
    tint[a, c] += p.tint
    tint[b, c] -= p.tint
    tint[a, e] -= p.tint
    tint[b, e] += p.tint
for i in X:
    for (j, next) in Y.all.adjacent: tint[i, next] += tint[i, j]
for j in Y:
    for (i, next) in X.all.adjacent: tint[next, j] += tint[i, j]

print(sum((xs.get(i2) - xs.get(i)) * (ys.get(j2) - ys.get(j))
          for (i, i2) in X.all.adjacent for (j, j2) in Y.all.adjacent if tint[i, j] >= need))
