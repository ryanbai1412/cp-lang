// IMO 2022 P6 (checker): a Nordic square holds 1..n^2 once each. An uphill
// path starts at a valley (a cell with no smaller neighbour) and climbs
// through side-adjacent cells. Count the uphill paths, mod 998244353.
// Runnable on proto/cpc today.
// original: corpus/submissions/4955728.cpp
let n: 1..=1500 = read()
domain Row = 0..n
domain Col = 0..n
type Cell = (Row, Col)
type M = Mod<998244353>
let a: [Row, Col]Int = read()

fn adj4(p: Cell) -> List<Cell>:  // v2: p.adj4
    var out: List<Cell> = []
    if let r = p.0 - 1: out.push((r, p.1))
    if let r = p.0 + 1: out.push((r, p.1))
    if let c = p.1 - 1: out.push((p.0, c))
    if let c = p.1 + 1: out.push((p.0, c))
    out

var at: [1..=n * n]Cell = (Row.first, Col.first)
for r in Row:
    for c in Col: at[a[r, c]] = (r, c)

// Paths ending at a cell, taken in increasing order of value.
var ways: [Row, Col]M = 0
var total: M = 0
for v in 1..=n * n:
    let p = at[v]
    var valley = true
    for q in adj4(p):
        if a[q] < v:
            valley = false
            ways[p] += ways[q]
    if valley: ways[p] = 1
    total += ways[p]
print(total)
