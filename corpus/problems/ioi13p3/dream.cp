// IOI 2013 P3 — Wombats: dynamic `R x C` grid shortest paths (`R <= 5000, C <= 200`)
// supporting horizontal/vertical edge weight updates and top-row-to-bottom-row queries.
//
// Group consecutive rows into blocks of `B = 10` rows and store each block's
// `C x C` min-plus transition matrix as a leaf of a `SegTree<Block, Matrix>`
// (min-plus multiplication is associative, with `0` on the diagonal and `none`
// elsewhere as identity). Grouping `B = 10` rows cuts the number of leaves and
// memory by 10x to fit the 256 MB IOI limit while keeping a single-leaf rebuild
// `O(B * C^2)` cheaper than the `O(C^2 log(R / B))` internal merges.
//
// Planar-grid crossing monotonicity (Knuth optimization): shortest paths from
// top column `i` to bottom column `j` need not cross, so the optimal middle
// column `opt[i, j]` satisfies `opt[i, j - 1] <= opt[i, j] <= opt[i + 1, j]`.
// Sweeping `i` downwards and `j` upwards therefore computes `mul(x, y)` in
// `O(C^2)` instead of `O(C^3)`. Tropical `?Int` (`+`, `min=`) removes every
// `1e9` sentinel.

const B = 10
type Matrix = List<List<?Int>>

fn identity(c: Int) -> Matrix:
    var out = List.filled(c, List.filled(c, None))
    for i in 0..c: out[i][i] = Some(0)
    out

fn mul(x: Matrix, y: Matrix) -> Matrix:
    let c = x.len
    var out = List.filled(c, List.filled(c, None))
    var opt = List.filled(c, List.filled(c, 0))
    for i in (0..c).rev:
        for j in 0..c:
            let lo = if j > 0: opt[i][j - 1] else: 0
            let hi = if i + 1 < c: opt[i + 1][j] else: c - 1
            var best: ?Int = None
            var best_k = lo
            for k in lo..=hi:
                if let cand = x[i][k] + y[k][j]:
                    if best.all(b => cand < b):
                        best = Some(cand)
                        best_k = k
            out[i][j] = best
            opt[i][j] = best_k
    out

@grader
struct Wombats:
    r: Int
    c: Int
    h: List<List<Int>>
    v: List<List<Int>>
    domain Block = 0..ceil_div(r, B)
    seg: SegTree<Block, Matrix>

    fn solve_block(self, blk: Block) -> Matrix:
        let lo = blk.ord * B
        let hi = min(self.r, lo + B)
        var out = identity(self.c)
        for i in 0..self.c:
            for row in lo..hi:
                for k in 1..self.c: out[i][k] min= out[i][k - 1] + self.h[row][k - 1]
                for k in (0..self.c - 1).rev: out[i][k] min= out[i][k + 1] + self.h[row][k]
                if row + 1 < self.r:
                    for k in 0..self.c: out[i][k] = out[i][k] + self.v[row][k]
        out

    fn init(r: Int, c: Int, h: List<List<Int>>, v: List<List<Int>>) -> Wombats:
        let init_leaves = [Block] b => self.solve_block(b)
        Wombats(r, c, h, v, SegTree<Block, Matrix>.new(Monoid(identity(c), mul), a: init_leaves))

    fn changeH(mut self, p: Int, q: Int, cost: Int):
        self.h[p][q] = cost
        let blk = Block(p / B)
        self.seg.set(blk, self.solve_block(blk))

    fn changeV(mut self, p: Int, q: Int, cost: Int):
        self.v[p][q] = cost
        let blk = Block(p / B)
        self.seg.set(blk, self.solve_block(blk))

    fn escape(self, v1: Int, v2: Int) -> Int = self.seg.all[v1][v2]!
