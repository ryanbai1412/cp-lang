// IMO 2022 P6 (checker): a Nordic square holds 1..n^2 once each. An uphill
// path starts at a valley (a cell with no smaller neighbour) and climbs
// through side-adjacent cells. Count the uphill paths, mod 998244353.
// original: corpus/submissions/4955728.cpp
let n: 1..=1500 = read()
domain Row = 0..n
domain Col = 0..n
type Cell = (Row, Col)
type Z = Mod<998244353>
let a: [Row, Col]Int = read()

// Paths ending at a cell, taken in increasing order of value.
var ways: [Row, Col]Z = 0
for p in sorted(Cell, by: p => a[p]):
    let lower = [q for q in p.adj4 if a[q] < a[p]]
    ways[p] = if lower.is_empty: 1 else: sum(ways[q] for q in lower)
print(sum(ways))
