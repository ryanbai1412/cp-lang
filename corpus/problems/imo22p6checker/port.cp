// Source: IMO 2022 P6 (checker), https://dmoj.ca/problem/imo22p6checker
// Problem: an n x n Nordic square (n <= 1500) holds 1..=n^2 once each. An uphill path
//   starts at a valley (a cell with no smaller neighbour) and climbs through side-
//   adjacent cells. Count the uphill paths, mod 998244353.
// Solution: take cells in increasing value: the paths ending at a cell are 1 at a
//   valley, else the sum over its smaller neighbours. Sum over all cells, O(n^2).
// original: corpus/submissions/4955728.cpp
// Bounds: n: 1..=1500; a: 1..=n * n
let n: 1..=1500 = read()
domain Row = 0..n
domain Col = 0..n
domain Value = 1..=n * n
type Cell = (Row, Col)
type M = Mod<998244353>
let a: [Row, Col]Value = read()
var at: [Value]Cell = (Row.first!, Col.first!)
for p in a.keys: at[a[p]] = p

// Paths ending at a cell, taken in increasing order of value: 1 at a valley, else the sum below.
var ways: [Row, Col]M = 0
for p in at:
    let lower = [q for q in p.adj4 if a[q] < a[p]]
    ways[p] = if lower.is_empty: M.one else: lower.sum(q => ways[q])
print(ways.sum())
