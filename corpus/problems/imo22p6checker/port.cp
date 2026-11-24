// Source: IMO 2022 P6 (checker), https://dmoj.ca/problem/imo22p6checker
// Problem: an n x n Nordic square (n <= 1500) holds 1..=n^2 once each. An uphill path
//   starts at a valley (a cell with no smaller neighbour) and climbs through side-
//   adjacent cells. Count the uphill paths, mod 998244353.
// Solution: take cells in increasing value: the paths ending at a cell are 1 at a
//   valley, else the sum over its smaller neighbours. Sum over all cells, O(n^2).
// original: corpus/submissions/4955728.cpp
let n: 1..=1500 = read()
domain Row = 0..n
domain Col = 0..n
type Cell = (Row, Col)
type M = Mod<998244353>
let a: [Row, Col]Int = read()

var at: [1..=n * n]Cell = (Row.first!, Col.first!)
for r in Row:
    for c in Col: at[a[r, c]] = (r, c)

// Paths ending at a cell, taken in increasing order of value.
var ways: [Row, Col]M = 0
var total: M = 0
for v in 1..=n * n:
    let p = at[v]
    var valley = true
    for q in p.adj4:
        if a[q] < v:
            valley = false
            ways[p] += ways[q]
    if valley: ways[p] = 1
    total += ways[p]
print(total)
