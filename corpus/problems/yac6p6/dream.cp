// YAC 6 P6: `n x m` grid (`n, m <= 200`); answer `q` queries `(x1, y1, x2, y2, kv)`
// (`0 <= kv <= 5`) for the minimum-cost down/right path from `(x1, y1)` to `(x2, y2)`
// allowed to skip (pay 0 for) at most `kv` cells.
//
// Diagonal meet-in-the-middle, pivot diagonals every `L` steps:
//   - Let `d` be the largest multiple of `L` with `d <= x2 + y2`. If `d < x1 + y1`, no
//     pivot diagonal lies between the ends, so one backward DP over the (at most
//     `L x L`) rectangle answers the query directly.
//   - Otherwise every path crosses the anti-diagonal `r + c = d` at exactly one cell, so
//     the query joins the bucket of each such cell inside its rectangle.
//   - Each non-empty bucket runs one backward DP from its queries' smallest start to the
//     pivot and one forward DP from the pivot to their largest end, and every query in it
//     splits its skips between the two halves.

const MAXK = 5
const L = 16
domain KDom = 0..=MAXK
type Cell = [KDom]?Int

let (n, m, q): (1..=200, 1..=200, 1..=200_000) = read()
domain Row = 1..=n
domain Col = 1..=m
domain Q = 0..q
let a: [Row, Col](1..=1e9) = read()

struct Query(x1: Row, y1: Col, x2: Row, y2: Col, kv: KDom, id: Q)
var qans: [Q]?Int = None
var buckets: [Row, Col]List<Query> = []
// back[r, c, k]: the cheapest walk from (r, c) to the pivot, both paid, skipping at most k cells.
var back: [Row, Col]Cell = [KDom] _ => None
// ahead[r, c, k]: the cheapest walk from just after the pivot to (r, c), skipping at most k cells.
var ahead: [Row, Col]Cell = [KDom] _ => None

// Through neighbour `next`: pay this cell's `cost` keeping `k` skips, or skip it with one of them.
fn via(next: Cell, k: KDom, cost: Int) -> ?Int:
    var best = next[k] + cost
    if let pk = k.prev: best min= next[pk]
    best

fn back_dp(x1: Row, y1: Col, x2: Row, y2: Col):
    for r in (x1..=x2).rev:
        for c in (y1..=y2).rev:
            let cost = a[r, c]
            for k in KDom:
                if r == x2 and c == y2:
                    back[r, c, k] = if k == KDom.first: Some(cost) else: Some(0)
                    continue
                var best: ?Int = None
                if let nr = r.next and nr <= x2: best min= via(back[nr, c], k, cost)
                if let nc = c.next and nc <= y2: best min= via(back[r, nc], k, cost)
                back[r, c, k] = best

fn ahead_dp(x1: Row, y1: Col, x2: Row, y2: Col):
    for r in x1..=x2:
        for c in y1..=y2:
            let cost = a[r, c]
            for k in KDom:
                if r == x1 and c == y1:
                    ahead[r, c, k] = Some(0)
                    continue
                var best: ?Int = None
                if let pr = r.prev and pr >= x1: best min= via(ahead[pr, c], k, cost)
                if let pc = c.prev and pc >= y1: best min= via(ahead[r, pc], k, cost)
                ahead[r, c, k] = best

for id in Q:
    let (x1, y1, x2, y2, kv): (Row, Col, Row, Col, KDom) = read()
    let d = Int(x2) + Int(y2) - (Int(x2) + Int(y2)) % L
    if d < Int(x1) + Int(y1):
        back_dp(x1, y1, x2, y2)
        qans[id] = back[x1, y1, kv]
    else:
        for x in max(Int(x1), d - Int(y2))..=min(Int(x2), d - Int(y1)):
            buckets[Row(x), Col(d - x)].push(Query(x1, y1, x2, y2, kv, id))

for cx in Row:
    for cy in Col if not buckets[cx, cy].is_empty:
        let bucket = buckets[cx, cy]
        back_dp(bucket.map(b => b.x1).min()!, bucket.map(b => b.y1).min()!, cx, cy)
        ahead_dp(cx, cy, bucket.map(b => b.x2).max()!, bucket.map(b => b.y2).max()!)
        for qr in bucket:
            for k in KDom if k <= qr.kv:
                qans[qr.id] min= back[qr.x1, qr.y1, k] + ahead[qr.x2, qr.y2, qr.kv.offset(-k.ord)!]

for id in Q: print(qans[id]!)
