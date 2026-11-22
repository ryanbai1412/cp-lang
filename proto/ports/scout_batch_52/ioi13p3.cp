// IOI 2013 P3 — Wombats: dynamic `R x C` grid shortest paths (`R <= 5000, C <= 200`)
// supporting horizontal/vertical edge weight updates and top-row-to-bottom-row queries.
//
// Group consecutive rows into blocks of `B = 10` rows and store each block's
// `C x C` min-plus transition matrix as a leaf of a `SegTree<Block, Matrix>`
// (min-plus multiplication is associative, with `0` on the diagonal and `none`
// elsewhere as identity). Grouping `B = 10` rows cuts leaf count and memory by
// 10x to fit the 256 MB IOI limit while keeping a single-leaf rebuild `O(B * C^2)`
// cheaper than the `O(C^2 log(R / B))` internal merges.
//
// Planar-grid crossing monotonicity (Knuth optimization): shortest paths from
// top column `i` to bottom column `j` need not cross, so the optimal middle
// column `opt[i, j]` satisfies `opt[i, j - 1] <= opt[i, j] <= opt[i + 1, j]`.
// Sweeping `i` downwards and `j` upwards computes `mul(x, y)` in `O(C^2)`
// instead of `O(C^3)`. Tropical `?Int` (`+`, `min=`) removes every `1e9` sentinel.

const B = 10
type Matrix = List<List<?Int>>

fn identity(cols: Int) -> Matrix:
    var out = List.filled(cols, List.filled(cols, none))
    for i in 0..cols: out[i][i] = some(0)
    out

fn mul(x: Matrix, y: Matrix) -> Matrix:
    let cols = x.len
    var out = List.filled(cols, List.filled(cols, none))
    var opt = List.filled(cols, List.filled(cols, 0))
    for i in (0..cols).rev:
        for j in 0..cols:
            let lo = if j > 0: opt[i][j - 1] else: 0
            let hi = if i + 1 < cols: opt[i + 1][j] else: cols - 1
            var best: ?Int = none
            var best_k = lo
            for k in lo..=hi:
                if let cand = x[i][k] + y[k][j]:
                    let better = if let b = best: cand < b else: true
                    if better:
                        best = some(cand)
                        best_k = k
            out[i][j] = best
            opt[i][j] = best_k
    out

let (r, c): (1..=5_000, 1..=200) = read()
var h: List<List<Int>> = [[read<Int>() for _ in 0..c - 1] for _ in 0..r]
var v: List<List<Int>> = [[read<Int>() for _ in 0..c] for _ in 0..r - 1]

domain Block = 0..ceil_div(r, B)

fn solve_block(blk: Block) -> Matrix:
    let lo = blk.ord * B
    let hi = min(r, lo + B)
    var out = identity(c)
    for row in lo..hi:
        let hr = h[row]
        for i in 0..c:
            for k in 1..c: out[i][k] min= out[i][k - 1] + hr[k - 1]
            for k in (0..c - 1).rev: out[i][k] min= out[i][k + 1] + hr[k]
        if row + 1 < r:
            let vr = v[row]
            for i in 0..c:
                for k in 0..c: out[i][k] = out[i][k] + vr[k]
    out

var seg = SegTree<Block, Matrix>.new(
    Monoid(identity(c), (a, b) => mul(a, b)),
    a: [Block] b => solve_block(b),
)

repeat read<Int>():
    match read<Int>():
        1 =>
            let (p, col, cost): (Int, Int, Int) = read()
            h[p][col] = cost
            let blk = Block(p / B)
            seg.set(blk, solve_block(blk))
        2 =>
            let (p, col, cost): (Int, Int, Int) = read()
            v[p][col] = cost
            let blk = Block(p / B)
            seg.set(blk, solve_block(blk))
        _ =>
            let (v1, v2): (Int, Int) = read()
            print(seg.all[v1][v2]!)
