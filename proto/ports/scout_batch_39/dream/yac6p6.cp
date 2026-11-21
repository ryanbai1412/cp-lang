// YAC 6 P6: `n x m` grid (`n, m <= 200`); answer `q` queries `(x1, y1, x2, y2, kv)`
// (`0 <= kv <= 5`) for the minimum-cost down/right path from `(x1, y1)` to `(x2, y2)`
// allowed to skip (pay 0 for) at most `kv` cells.
//
// Diagonal meet-in-the-middle:
//   - If `x2 - x1 <= 3` and `y2 - y1 <= 3`, the rectangle has at most 16 cells, so
//     we run `backward_dp(x1, y1, x2, y2)` directly on it.
//   - Otherwise, let `rem = (x2.ord + y2.ord) % 3`. Every path from `(x1, y1)` to
//     `(x2, y2)` crosses the anti-diagonal `r.ord + c.ord = x2.ord + y2.ord - rem`
//     at some cell `(cx, cy)` with `cx in x1..=x2`, `cy in y1..=y2`, and
//     `(x2 - cx) + (y2 - cy) = rem <= 2` (so `(x2, y2)` lies inside the `3 x 3` box
//     below-right of `(cx, cy)`).
//   - Bucketing queries by `(cx, cy)` (only cells with `(cx.ord + cy.ord) % 3 == 0`
//     and at least one query are computed), each populated bucket runs one `3 x 3`
//     forward suffix DP and one `cx x cy` backward prefix DP; summing `cx * cy`
//     over non-empty buckets gives at most `(n^2 * m^2 / 12) * (MAXK + 1) + q * (MAXK + 1)`
//     cell-state steps (and far fewer when queries touch few buckets).

const MAXK = 5
domain KDom = 0..=MAXK
domain D3 = 0..3
type Cell = [KDom]?Int

let (n, m, q): (1..=200, 1..=200, 1..=200_000) = read()
domain Row = 1..=n
domain Col = 1..=m
domain Q = 0..q
let a: [Row, Col]Int = read()

struct Query(x1: Row, y1: Col, x2: Row, y2: Col, kv: KDom, id: Q)
var qans: [Q]?Int = none
var buckets: [Row, Col]List<Query> = []
let empty_cell: Cell = [KDom] _ => none
var dp: [Row, Col]Cell = empty_cell
var dp2: [D3, D3]Cell = empty_cell

// Relax a cell's skip-cost table from one neighbour: either pay `cost` with the
// same skip count `k`, or skip this cell using `k - 1` skips from the neighbour.
fn relax(cur: Cell, from: Cell, cost: Int) -> Cell:
    var out = cur
    for k in KDom:
        out[k] min= from[k] + cost
        if let pk = k.prev: out[k] min= from[pk]
    out

fn backward_dp(x1: Row, y1: Col, x2: Row, y2: Col):
    for r in (x1..=x2).rev:
        for c in (y1..=y2).rev:
            let cost = a[r, c]
            if r == x2 and c == y2:
                dp[r, c] = [KDom] k => if k == KDom.first: some(cost) else: some(0)
            else:
                var cell: Cell = [KDom] _ => none
                if let nr = r.next and nr <= x2: cell = relax(cell, dp[nr, c], cost)
                if let nc = c.next and nc <= y2: cell = relax(cell, dp[r, nc], cost)
                dp[r, c] = cell

for id in Q:
    let (x1, y1, x2, y2, kv): (Row, Col, Row, Col, KDom) = read()
    if x2.ord - x1.ord <= 3 and y2.ord - y1.ord <= 3:
        backward_dp(x1, y1, x2, y2)
        qans[id] = dp[x1, y1][kv]
    else:
        let rem = (x2.ord + y2.ord) % 3
        for dx in 0..=rem:
            let dy = rem - dx
            if let cx = x2.offset(-dx), let cy = y2.offset(-dy) and cx >= x1 and cy >= y1:
                buckets[cx, cy].push(Query(x1, y1, x2, y2, kv, id))

for cx in Row:
    for cy in Col if not buckets[cx, cy].is_empty:
        // Forward 3x3 suffix DP starting after (cx, cy).
        for dr in D3:
            for dc in D3:
                if let r = cx.offset(dr.ord), let c = cy.offset(dc.ord):
                    if dr == D3.first and dc == D3.first:
                        dp2[dr, dc] = [KDom] _ => some(0)
                    else:
                        let cost = a[r, c]
                        var cell: Cell = [KDom] _ => none
                        if let pr = dr.prev: cell = relax(cell, dp2[pr, dc], cost)
                        if let pc = dc.prev: cell = relax(cell, dp2[dr, pc], cost)
                        dp2[dr, dc] = cell

        backward_dp(Row.first, Col.first, cx, cy)

        for qr in buckets[cx, cy]:
            let dr = D3.first.offset(qr.x2.ord - cx.ord)!
            let dc = D3.first.offset(qr.y2.ord - cy.ord)!
            for k in 0..=qr.kv.ord:
                qans[qr.id] min= dp[qr.x1, qr.y1][KDom(k)] + dp2[dr, dc][qr.kv.offset(-k)!]

for id in Q: print(qans[id]!)
