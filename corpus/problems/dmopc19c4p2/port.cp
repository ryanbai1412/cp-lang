// DMOPC '19 Contest 4 P2: a token and a hole share an n x n board (the 15
// puzzle with one tile that matters). One move slides the hole one step, or
// swaps the token into the hole when they are adjacent. Fewest moves until
// the token stands on the target.
// Runnable on proto/cpc today.
// original: corpus/submissions/5323330.cpp
let n: 2..=16 = read()
domain Row = 1..=n
domain Col = 1..=n
type Cell = (Row, Col)
let s: Cell = read()
let t: Cell = read()
let e1: Cell = read()
let e2: Cell = read()

fn adj4(p: Cell) -> List<Cell>:  // v2: p.adj4
    var out: List<Cell> = []
    if let r = p.0 - 1: out.push((r, p.1))
    if let r = p.0 + 1: out.push((r, p.1))
    if let c = p.1 - 1: out.push((p.0, c))
    if let c = p.1 + 1: out.push((p.0, c))
    out

// The token starts on one of the two given cells; the hole is the other.
let hole0 = if s == e1: e2 else: e1
var dist: [Row, Col, Row, Col]?Int = none
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
    for h2 in adj4(h):
        if h2 == p: next.push((h, p))  // token slides into the hole
        else: next.push((p, h2))
    for st in next:
        let a = st.0
        let b = st.1
        if dist[a.0, a.1, b.0, b.1] == none:
            dist[a.0, a.1, b.0, b.1] = d + 1
            q.push(st)
print(ans)
