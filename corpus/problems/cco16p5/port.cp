// Source: CCO 2016 Day 2 P2 (Zombie Apocalypse), https://dmoj.ca/problem/cco16p5
// Problem: an N x M grid (N, M <= 1e9) holds K <= 2000 zombies in distinct cells. Each cell
//   is marked with its king-move (Chebyshev) distance to the nearest zombie. Given Q
//   (0 <= Q <= N + M), print how many cells are marked Q.
// Solution: the cells within distance Q are the union of the K squares of radius Q
//   clipped to the grid, so the answer is area(Q) - area(Q - 1). Cut the columns at
//   every square's edges; within one strip the covered rows are the union of the squares
//   spanning it, measured by one pass over the squares sorted by top row: O(K^2) per area.
// original: corpus/submissions/4580695.cpp
// Bounds: n, m: 1..=1e9; k: 1..=2000; zombies.r: 1..=n; zombies.c: 1..=m; q: 0..=n + m
let n: 1..=1e9 = read()
let m: 1..=1e9 = read()
let k: 1..=2000 = read()
let zombies: [k](r: 1..=n, c: 1..=m) = read()
let q: 0..=n + m = read()

// The number of cells within distance rad of a zombie: the union of the clipped squares.
fn covered(rad: Int) -> Int:
    var squares = [(r0: max(1, z.r - rad), r1: min(n, z.r + rad) + 1,
                    c0: max(1, z.c - rad), c1: min(m, z.c + rad) + 1) for z in zombies]
    squares.sort(by: s => s.r0)
    var cuts = [s.c0 for s in squares] + [s.c1 for s in squares]
    cuts.sort()
    cuts.dedup()
    var area = 0
    for (lo, hi) in cuts.adjacent:
        var (rows, reach) = (0, 1)  // rows counted so far, all of them above `reach`
        for s in squares:
            if s.c0 <= lo and hi <= s.c1 and s.r1 > reach:
                rows += s.r1 - max(s.r0, reach)
                reach = s.r1
        area += (hi - lo) * rows
    area

print(covered(q) - (if q == 0: 0 else: covered(q - 1)))
