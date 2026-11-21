// YAC 6 P6: `n x m` grid (`n, m <= 200`); answer `q` queries `(x1, y1, x2, y2, kv)`
// (`0 <= kv <= 5`) for the minimum-cost down/right path from `(x1, y1)` to `(x2, y2)`
// allowed to skip (pay 0 for) at most `kv` cells.
//
// Diagonal meet-in-the-middle:
//   - If `x2 - x1 <= 3` and `y2 - y1 <= 3`, the rectangle has at most 16 cells, so
//     we run the DP directly on it.
//   - Otherwise, let `rem = (x2.ord + y2.ord) % 3`. Every path from `(x1, y1)` to
//     `(x2, y2)` crosses the anti-diagonal `r.ord + c.ord = x2.ord + y2.ord - rem`
//     at some cell `(cx, cy)` with `cx in x1..=x2`, `cy in y1..=y2`, and
//     `(x2 - cx) + (y2 - cy) = rem <= 2` (so `(x2, y2)` lies inside the `3 x 3` box
//     below-right of `(cx, cy)`). Only one-third of all grid cells have
//     `(cx.ord + cy.ord) % 3 == 0`, so bucketing queries by `(cx, cy)` and computing
//     one backward prefix DP up to `(cx, cy)` plus one `3 x 3` forward suffix DP
//     starting after `(cx, cy)` runs in `O((nm / 3) * nm * MAXK + q * MAXK)`.

const MAXK = 5
domain KDom = 0..=MAXK
domain D3 = 0..3

let (n, m, q): (1..=200, 1..=200, 1..=200_000) = read()
domain Row = 1..=n
domain Col = 1..=m
domain Q = 0..q
let a: [Row, Col]Int = read()

struct Query(x1: Row, y1: Col, x2: Row, y2: Col, kv: KDom, id: Q)
var qans: [Q]?Int = none
var buckets: [Row, Col]List<Query> = []

// One min-plus cell step: either pay `cost` with the same skip count, or skip the cell (`k - 1`).
fn step_min(best: ?Int, from_same: ?Int, from_skip: ?Int, cost: Int) -> ?Int:
    var res = best
    res min= from_same + cost
    res min= from_skip
    res

fn small_dp(x1: Row, y1: Col, x2: Row, y2: Col, kv: KDom) -> Int:
    let (h, w) = (x2.ord - x1.ord + 1, y2.ord - y1.ord + 1)
    var sub: [0..h, 0..w, KDom]?Int = none
    for i in (0..h).rev:
        for j in (0..w).rev:
            let cost = a[Row(x1.ord + i + 1), Col(y1.ord + j + 1)]
            if i == h - 1 and j == w - 1:
                sub[i, j, KDom(0)] = cost
                for k in KDom if k > KDom(0): sub[i, j, k] = 0
            else:
                for k in KDom:
                    var best: ?Int = none
                    if i + 1 < h:
                        let skip: ?Int = if let pk = k.prev: sub[i + 1, j, pk] else: none
                        best = step_min(best, sub[i + 1, j, k], skip, cost)
                    if j + 1 < w:
                        let skip: ?Int = if let pk = k.prev: sub[i, j + 1, pk] else: none
                        best = step_min(best, sub[i, j + 1, k], skip, cost)
                    sub[i, j, k] = best
    sub[0, 0, kv]!

for id in Q:
    let (x1, y1, x2, y2, kv): (Row, Col, Row, Col, KDom) = read()
    if x2.ord - x1.ord <= 3 and y2.ord - y1.ord <= 3:
        qans[id] = small_dp(x1, y1, x2, y2, kv)
    else:
        let rem = (x2.ord + y2.ord) % 3
        for dx in 0..=rem:
            let dy = rem - dx
            if let cx = x2.offset(-dx), let cy = y2.offset(-dy), cx >= x1 and cy >= y1:
                buckets[cx, cy].push(Query(x1, y1, x2, y2, kv, id))

var dp: [Row, Col, KDom]?Int = none
var dp2: [D3, D3, KDom]?Int = none

for cx in Row:
    for cy in Col if not buckets[cx, cy].is_empty:
        // Forward 3x3 suffix DP starting after (cx, cy).
        for dr in D3:
            for dc in D3:
                if let r = cx.offset(dr.ord), let c = cy.offset(dc.ord):
                    if dr == D3.first and dc == D3.first:
                        for k in KDom: dp2[dr, dc, k] = 0
                    else:
                        let cost = a[r, c]
                        for k in KDom:
                            var best: ?Int = none
                            if let pr = dr.prev:
                                let skip: ?Int = if let pk = k.prev: dp2[pr, dc, pk] else: none
                                best = step_min(best, dp2[pr, dc, k], skip, cost)
                            if let pc = dc.prev:
                                let skip: ?Int = if let pk = k.prev: dp2[dr, pc, pk] else: none
                                best = step_min(best, dp2[dr, pc, k], skip, cost)
                            dp2[dr, dc, k] = best

        // Backward prefix DP up to (cx, cy) inclusive.
        for r in (Row.first..=cx).rev:
            for c in (Col.first..=cy).rev:
                if r == cx and c == cy:
                    dp[r, c, KDom(0)] = a[r, c]
                    for k in KDom if k > KDom(0): dp[r, c, k] = 0
                else:
                    let cost = a[r, c]
                    for k in KDom:
                        var best: ?Int = none
                        if let nr = r.next, nr <= cx:
                            let skip: ?Int = if let pk = k.prev: dp[nr, c, pk] else: none
                            best = step_min(best, dp[nr, c, k], skip, cost)
                        if let nc = c.next, nc <= cy:
                            let skip: ?Int = if let pk = k.prev: dp[r, nc, pk] else: none
                            best = step_min(best, dp[r, nc, k], skip, cost)
                        dp[r, c, k] = best

        for qr in buckets[cx, cy]:
            let dr = D3.first.offset(qr.x2.ord - cx.ord)!
            let dc = D3.first.offset(qr.y2.ord - cy.ord)!
            for k in 0..=qr.kv.ord:
                qans[qr.id] min= dp[qr.x1, qr.y1, KDom(k)] + dp2[dr, dc, qr.kv.offset(-k)!]

for id in Q: print(qans[id]!)
