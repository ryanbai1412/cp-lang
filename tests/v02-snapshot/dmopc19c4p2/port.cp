// Source: DMOPC '19 Contest 4 P2 (Pleasant Present), https://dmoj.ca/problem/dmopc19c4p2
// Problem: an N x N grid, N <= 15, where all but 2 cells hold magnets; a marble starts on one
//   empty cell. Each second either the marble moves to an adjacent empty cell or a magnet slides
//   into the adjacent empty cell. Print the fewest seconds to get the marble to (r_e, c_e).
// Solution: the state is (marble cell, hole cell), the 15 puzzle with one tile that matters.
//   A move slides the hole one step, or swaps marble and hole when they are adjacent. BFS over
//   the N^4 states with a [Row, Col, Row, Col] distance table: O(N^4).
// original: corpus/submissions/5323330.cpp
let n: 2..=16 = read()
domain Row = 1..=n
domain Col = 1..=n
type Cell = (Row, Col)
let s: Cell = read()
let t: Cell = read()
let e1: Cell = read()
let e2: Cell = read()

// The token starts on one of the two given cells; the hole is the other.
let hole0 = if s == e1: e2 else: e1
var dist: [Row, Col, Row, Col]?Int = None
var q: List<(Cell, Cell)> = [(s, hole0)]
dist[s.0, s.1, hole0.0, hole0.1] = 0
var head = 0
var ans = -1
while head < q.len:
    let cur = q[head]
    head += 1
    let p = cur.0
    let h = cur.1
    let d = dist[p.0, p.1, h.0, h.1]!
    if p == t:
        ans = d
        break
    var next: List<(Cell, Cell)> = []
    for h2 in h.adj4:
        if h2 == p: next.push((h, p))  // token slides into the hole
        else: next.push((p, h2))
    for st in next:
        let a = st.0
        let b = st.1
        if dist[a.0, a.1, b.0, b.1] == None:
            dist[a.0, a.1, b.0, b.1] = d + 1
            q.push(st)
print(ans)
