// IOI 2013 P3 — Wombats: dynamic `R x C` grid shortest paths (`R <= 5000, C <= 200`)
// supporting horizontal/vertical edge weight updates and top-row-to-bottom-row queries.
//
// Group consecutive rows into blocks of `B = 20` rows and store each block's
// `C x C` min-plus transition matrix as a leaf of a `SegTree<Block, Matrix>`
// (min-plus multiplication is associative, with `0` on the diagonal and `None`
// elsewhere as identity); 20-row blocks keep the tree's matrices within 256 MB. A change rebuilds one leaf in `O(B * C^2)` and its
// `O(log(R / B))` ancestors.
//
// Planar-grid crossing monotonicity (Knuth optimization): shortest paths from
// top column `i` to bottom column `j` need not cross, so the optimal middle
// column `opt[i, j]` satisfies `opt[i, j - 1] <= opt[i, j] <= opt[i + 1, j]`.
// Sweeping `i` downwards and `j` upwards therefore computes `mul(x, y)` in
// `O(C^2)` instead of `O(C^3)`. Tropical `?Cost` (`+`, `min=`) removes every
// `1e9` sentinel.
const B = 20
type Wombats = 0..=1000
type Cost = 0..=1e7  // a route crosses at most R * 1000 + C * 1000 wombats

let r: 1..=5000 = read()
let c: 1..=200 = read()
domain Row = 0..r
domain Col = 0..c
domain Gap = 0..c - 1
domain Drop = 0..r - 1
var h: [Row, Gap]Wombats = read()
var v: [Drop, Col]Wombats = read()
domain Block = 0..ceil_div(r, B)
type Matrix = [Col, Col]?Cost

let identity: Matrix = [Col, Col] (i, j) => if i == j: Some(0) else: None

fn mul(x: Matrix, y: Matrix) -> Matrix:
    var out: Matrix = None
    var opt: [Col, Col]Col = Col.first!
    for i in Col.all.rev:
        for j in Col:
            let lo = j.prev.map(p => opt[i, p]) ?? Col.first!
            let hi = i.next.map(n => opt[n, j]) ?? Col.last!
            opt[i, j] = min(max(i, lo), hi)
            for k in lo..=hi:
                if let cand = x[i, k] + y[k, j]:
                    if out[i, j].map(best => cand < best) ?? true:
                        out[i, j] = Some(cand)
                        opt[i, j] = k
    out

fn solve_block(blk: Block) -> Matrix:
    let rows = Row.filter(row => row.ord / B == blk.ord)
    var out = identity
    for i in Col:
        for row in rows:
            for g in Gap: out[i, Col(g.ord + 1)] min= out[i, Col(g.ord)] + h[row, g]
            for g in Gap.all.rev: out[i, Col(g.ord)] min= out[i, Col(g.ord + 1)] + h[row, g]
            if let d = Drop.at(row.ord):
                for k in Col: out[i, k] = out[i, k] + v[d, k]
    out

var seg = SegTree<Block, Matrix>(Monoid(identity, mul), [Block] b => solve_block(b))
fn refresh(row: Int):
    let blk = Block(row / B)
    seg.set(blk, solve_block(blk))

let events: 1..=2e5 + 500 = read()
for _ in 0..events:
    let kind: 1..=3 = read()
    if kind == 1:
        let (p, q, w): (Row, Gap, Wombats) = read()
        h[p, q] = w
        refresh(p.ord)
    elif kind == 2:
        let (p, q, w): (Drop, Col, Wombats) = read()
        v[p, q] = w
        refresh(p.ord)
    else:
        let (v1, v2): (Col, Col) = read()
        print(seg.all[v1, v2]!)
