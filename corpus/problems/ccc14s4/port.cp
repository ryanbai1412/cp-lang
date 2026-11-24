// Source: CCC 2014 Senior S4 (Tinted Glass Window), https://dmoj.ca/problem/ccc14s4
// limits: 1000 ms, 256 MB
// Problem: n <= 1000 axis-parallel rectangles, corners x_l y_t x_r y_b in 0..=K with K <= 1e9,
//   each adding tint t_i; overlaps add up. Print the total area whose tint is at least
//   T <= 1e9 (the answer can exceed 2^32 but fits 64 bits).
// Solution: the 2n pane edges cut each axis into at most 2n - 1 strips, and the tint is constant
//   on every cell of that grid. Compress both axes, mark each pane's four corners in a
//   difference grid, take 2D prefix sums and add the area of every cell dark enough: O(n^2)
//   cells, no data structure (the original sweeps x over a sqrt-decomposed y axis).
// original: corpus/submissions/5672281.cpp
let n: Int = read()
let t: Int = read()
struct Pane(x0: Int, y0: Int, x1: Int, y1: Int, tint: Int)
var panes: List<Pane> = []
for _ in 0..n:
    let x0: Int = read()
    let y0: Int = read()
    let x1: Int = read()
    let y1: Int = read()
    let tint: Int = read()
    panes.push(Pane(x0, y0, x1, y1, tint))

// the distinct values, sorted; a value's position is its cut number
fn cuts(vals: List<Int>) -> List<Int>:
    var out: List<Int> = []
    for v in vals.sorted():
        if out.is_empty or out.last != v: out.push(v)
    out

fn rank(sorted_cuts: List<Int>, v: Int) -> Int:
    (0..sorted_cuts.len - 1).first_true(mid => sorted_cuts[mid] >= v) ?? (sorted_cuts.len - 1)

var (all_x, all_y): (List<Int>, List<Int>) = ([], [])
for p in panes:
    all_x.push(p.x0)
    all_x.push(p.x1)
    all_y.push(p.y0)
    all_y.push(p.y1)
let xs = cuts(all_x)
let ys = cuts(all_y)
domain X = 0..xs.len  // cell (i, j) is xs[i]..xs[i+1] by ys[j]..ys[j+1]
domain Y = 0..ys.len

var tint: [X, Y]Int = 0
for p in panes:
    let (a, b) = (X(rank(xs, p.x0)), X(rank(xs, p.x1)))
    let (c, e) = (Y(rank(ys, p.y0)), Y(rank(ys, p.y1)))
    tint[a, c] += p.tint
    tint[b, c] -= p.tint
    tint[a, e] -= p.tint
    tint[b, e] += p.tint

var dark = 0
for i in X:
    for j in Y:
        if let up = i.prev: tint[i, j] += tint[up, j]
        if let left = j.prev: tint[i, j] += tint[i, left]
        if let up = i.prev:
            if let left = j.prev: tint[i, j] -= tint[up, left]
        if tint[i, j] >= t:
            dark += (xs[i.ord + 1] - xs[i.ord]) * (ys[j.ord + 1] - ys[j.ord])
print(dark)
