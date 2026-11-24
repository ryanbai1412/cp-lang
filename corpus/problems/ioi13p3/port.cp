// Source: IOI 2013 P3 (Wombats), https://dmoj.ca/problem/ioi13p3
// Problem: an R x C grid of roads (R <= 5000, C <= 200) with at most 1000 wombats on each
//   segment; people walk east or west along rows and only south between them. Handle <= 500
//   changes of one segment and <= 2e5 escape(V1, V2) queries: the fewest wombats on a route from
//   (0, V1) to (R - 1, V2).
// Solution: cut the rows into blocks of 20 and keep each block's C x C matrix of least costs
//   from its top row to the row below it, built by C left/right relaxations per row in
//   O(10 C^2). A SegTree over blocks multiplies them in the (min, +) semiring, with `?Cost`
//   (None = no route) as its zero. Routes from neighbouring tops never need to cross, so the
//   best middle column is monotone (opt[i][j-1] <= k <= opt[i+1][j]) and a product is O(C^2).
//   A change rebuilds one block and its O(log R) ancestors; an escape reads the root.
// limits: 9000 ms, 256 MB
// Bounds: r: 1..=5000; c: 1..=200; east, south, w: 0..=1000; events: 1..=2e5 + 500; kind: 1..=3; row: 0..r; gap: 0..c - 1; drop: 0..r - 1; col, v1, v2: 0..c
// original: corpus/submissions/4722414.cpp (orig.cpp enables its sample grader)
const BLOCK = 20
type Wombats = 0..=1000
type Cost = 0..=1e7  // the cheapest route never exceeds R * 1000 + C * 1000

let r: 1..=5000 = read()
let c: 1..=200 = read()
domain Row = 0..r
domain Col = 0..c
domain Gap = 0..c - 1  // the road between columns g and g + 1
domain Drop = 0..r - 1  // the road below row d
var east: [Row, Gap]Wombats = read()
var south: [Drop, Col]Wombats = read()
domain Block = 0..ceil_div(r, BLOCK)

type Matrix = [Col, Col]?Cost

fn identity() -> Matrix = [Col, Col] (i, j) => if i == j: Some(0) else: None

fn mul(x: Matrix, y: Matrix) -> Matrix:
    var out: Matrix = None
    var opt: [Col, Col]Col = Col.first!
    for i in Col.all.rev:
        for j in Col:
            let lo = j.prev.map(p => opt[i, p]) ?? Col.first!
            let hi = i.next.map(n => opt[n, j]) ?? Col.last!
            opt[i, j] = min(max(i, lo), hi)  // where no route exists (the identity's off-diagonal)
            for k in lo..=hi:
                if let via = x[i, k] + y[k, j]:
                    if out[i, j].map(best => via < best) ?? true:
                        out[i, j] = Some(via)
                        opt[i, j] = k

    out

fn block(b: Block) -> Matrix:
    let rows = span(Row.cuts.nth(b.ord * BLOCK), Row.cuts.nth(min(r, (b.ord + 1) * BLOCK)))
    let top = rows.first!
    var along: [Col]Int = 0  // the cost along the top row from column 0
    for g in Gap: along[Col(g.ord + 1)] = along[Col(g.ord)] + east[top, g]
    var out: Matrix = None
    for s in Col:
        var cost = [Col] j => abs(along[j] - along[s])
        for row in rows:
            if row != top:
                for g in Gap: cost[Col(g.ord + 1)] min= cost[Col(g.ord)] + east[row, g]
                for g in Gap.all.rev: cost[Col(g.ord)] min= cost[Col(g.ord + 1)] + east[row, g]
            if let d = Drop.at(row.ord):
                for j in Col: cost[j] += south[d, j]
        for j in Col: out[s, j] = Some(cost[j])
    out

var routes = SegTree<Block, Matrix>(Monoid(identity(), mul), [Block] b => block(b))
fn block_of(row: Int) -> Block = Block(row / BLOCK)

let events: 1..=2e5 + 500 = read()
for _ in 0..events:
    let kind: 1..=3 = read()
    if kind == 1:
        let (row, gap, w): (Row, Gap, Wombats) = read()
        east[row, gap] = w
        routes.set(block_of(row.ord), block(block_of(row.ord)))
    elif kind == 2:
        let (drop, col, w): (Drop, Col, Wombats) = read()
        south[drop, col] = w
        routes.set(block_of(drop.ord), block(block_of(drop.ord)))
    else:
        let (v1, v2): (Col, Col) = read()
        print(routes.all[v1, v2]!)
