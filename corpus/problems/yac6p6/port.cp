// limits: 1000 ms, 256 MB
// Source: Yet Another Contest 6 P6 (No More Snakes and Ladders), https://dmoj.ca/problem/yac6p6
// Problem: an n x m grid (n, m <= 200) of values <= 1e9. Each of Q <= 1e5 independent queries
//   gives a subgrid (A, B)..(C, D) and K <= 5: a token walks right/down from (A, B) to (C, D)
//   and may first set up to K cells to 0; print the least possible sum of the visited cells.
// Solution: tables of the least cost per count of zeros used. Every path crosses the
//   anti-diagonal x + y = d, d the end's x + y rounded down to a multiple of L = 16, unless the
//   start is already past it, and then the subgrid is small enough to walk directly. Else
//   bucket the query at each cell of that diagonal inside its subgrid. Per pivot with queries,
//   one backward walk from the bucket's furthest-back start gives start-to-pivot tables, one
//   forward walk of under L x L cells the pivot-to-end tables, and each query takes the best
//   split of its zeros. About nm / L pivots, O(nm K) each; O(L^2 K) a direct query.
// original: corpus/submissions/5150025.cpp
// Bounds: n, m: 1..=200; q: 1..=200_000; a: 1..=1e9; x1, y1, x2, y2: Row/Col; zeros: Zeros
const MAXK = 5
const L = 16
domain Zeros = 0..=MAXK

let (n, m, q): (1..=200, 1..=200, 1..=200_000) = read()
domain Row = 1..=n
domain Col = 1..=m
domain Query = 0..q
type Cell = (Row, Col)
let a: [Row, Col](1..=1e9) = read()

// back[r, c, k]: from (r, c) to the walk's end, both counted, with at most k zeros
var back: [Row, Col, Zeros]Int = 0
// ahead[r, c, k]: from just after the walk's start to (r, c), with at most k zeros
var ahead: [Row, Col, Zeros]Int = 0

fn onward(r: Row, c: Col, to: Cell, k: Zeros) -> Int:
    if r == to.0: return back[r, c.next!, k]
    if c == to.1: return back[r.next!, c, k]
    min(back[r.next!, c, k], back[r, c.next!, k])

fn walk_back(from: Cell, to: Cell):
    for r in (from.0..=to.0).rev:
        for c in (from.1..=to.1).rev:
            let cost = a[r, c]
            for k in Zeros:
                if (r, c) == to:
                    back[r, c, k] = if k == Zeros.first: cost else: 0
                    continue
                var best = onward(r, c, to, k) + cost
                if let fewer = k.prev: best min= onward(r, c, to, fewer)
                back[r, c, k] = best

fn before(r: Row, c: Col, from: Cell, k: Zeros) -> Int:
    if r == from.0: return ahead[r, c.prev!, k]
    if c == from.1: return ahead[r.prev!, c, k]
    min(ahead[r.prev!, c, k], ahead[r, c.prev!, k])

fn walk_ahead(from: Cell, to: Cell):
    for r in from.0..=to.0:
        for c in from.1..=to.1:
            let cost = a[r, c]
            for k in Zeros:
                if (r, c) == from:
                    ahead[r, c, k] = 0
                    continue
                var best = before(r, c, from, k) + cost
                if let fewer = k.prev: best min= before(r, c, from, fewer)
                ahead[r, c, k] = best

type Ask = (from: Cell, to: Cell, zeros: Zeros, id: Query)
var answer: [Query]?Int = None
var buckets: [Row, Col]List<Ask> = []
for id in Query:
    let (x1, y1, x2, y2, zeros): (Row, Col, Row, Col, Zeros) = read()
    let d = Int(x2) + Int(y2) - (Int(x2) + Int(y2)) % L
    if d < Int(x1) + Int(y1):
        walk_back((x1, y1), (x2, y2))
        answer[id] = back[x1, y1, zeros]
    else:
        for x in max(Int(x1), d - Int(y2))..=min(Int(x2), d - Int(y1)):
            buckets[Row(x), Col(d - x)].push((from: (x1, y1), to: (x2, y2), zeros:, id:))

for pivot in (Row, Col):
    let bucket = buckets[pivot]
    if bucket.is_empty: continue
    walk_back((bucket.map(b => b.from.0).min()!, bucket.map(b => b.from.1).min()!), pivot)
    walk_ahead(pivot, (bucket.map(b => b.to.0).max()!, bucket.map(b => b.to.1).max()!))
    for (from, to, zeros, id) in bucket:
        for k in Zeros if k <= zeros:
            answer[id] min= back[from.0, from.1, k] + ahead[to.0, to.1, zeros.offset(-k.ord)!]

for id in Query: print(answer[id]!)
