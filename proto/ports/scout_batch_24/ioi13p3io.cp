// IOI '13 P3 - Wombats (Standard I/O) (DMOJ: ioi13p3io, 40pt, TL 9.0s, ML 256MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// R x C grid (R <= 5000, C <= 200) with horizontal road costs H[r, c]
// (0 <= r < R, 0 <= c < C-1) and vertical road costs V[r, c] (0 <= r < R-1, 0 <= c < C).
// Queries:
//   1 p q w   : H[p, q] = w
//   2 p q w   : V[p, q] = w
//   3 v1 v2   : shortest path from (0, v1) to (R-1, v2) moving down/left/right
//
// Block Segment Tree over C x C Monge min-plus matrices (block size B = 16 rows
// so tree nodes fit well inside 256 MB) with Knuth-accelerated O(C^2) combine.
use std.algebra.{Monoid}
use std.segtree.{SegTree}

let (r_cnt, c_cnt): (1..=5000, 1..=200) = read()
domain Row = 0..r_cnt
domain Col = 0..c_cnt
domain HCol = 0..(c_cnt - 1)
domain VRow = 0..(r_cnt - 1)

var h_cost: [Row, HCol]I32 = read()
var v_cost: [VRow, Col]I32 = read()
let e_cnt: 0..=200_500 = read()

enum Event:
    @tag("1") ChangeH(p: Row, q: HCol, w: I32)
    @tag("2") ChangeV(p: VRow, q: Col, w: I32)
    @tag("3") Escape(v1: Col, v2: Col)

let events: [0..e_cnt]Event = read()

// WISH A17: Heavy value-type `Monoid<Mat>` collision with `SegTree<D, T>`!
// `Monoid.op` takes `(Mat, Mat) -> Mat` by value, allocating a fresh 160 KB
// `C x C` matrix on every internal node merge and copying it into `t[p]`,
// whereas in-place `combine(a, b, mut out)` reuses `t[p]`'s buffer with zero
// heap churn. Also, `SegTree.set(i, build_block(i))` moves a whole matrix
// instead of mutating the leaf in place!
struct Mat(d: [Col, Col]I32)

const INF32: I32 = 1_000_000_000i32
const BLOCK: Int = 16

fn relax_row(row: Row, mut dist: [Col]I32):
    for c in 1..c_cnt:
        dist[Col(c)] min= dist[Col(c - 1)] + h_cost[row, HCol(c - 1)]
    for c in (0..(c_cnt - 1)).rev:
        dist[Col(c)] min= dist[Col(c + 1)] + h_cost[row, HCol(c)]

fn build_block(r_lo: Int, r_hi: Int) -> Mat:
    // Transitions from row r_lo to row min(r_hi + 1, r_cnt - 1); last block
    // also relaxes horizontal edges on row r_cnt - 1.
    var d: [Col, Col]I32 = [Col, Col] (i, j) => if i == j: 0i32 else: INF32
    let last_trans = r_cnt - 2
    for s in Col:
        var cur: [Col]I32 = [Col] j => if j == s: 0i32 else: INF32
        for r in r_lo..=r_hi:
            relax_row(Row(r), mut cur)
            if r <= last_trans:
                for j in Col:
                    cur[j] += v_cost[VRow(r), j]
        if r_hi == last_trans:
            relax_row(Row(r_cnt - 1), mut cur)
        for j in Col:
            d[s, j] = cur[j]
    Mat(d)

// Knuth-accelerated Monge min-plus product: opt[i, j-1] <= opt[i, j] <= opt[i+1, j].
// O(C^2) instead of O(C^3).
fn combine_monge(a: Mat, b: Mat) -> Mat:
    var out: [Col, Col]I32 = [Col, Col] (_, _) => INF32
    var opt: [Col, Col]Int = [Col, Col] (_, _) => 0
    for i in (0..c_cnt).rev:
        for j in 0..c_cnt:
            let lo = if j > 0: opt[Col(i), Col(j - 1)] else: 0
            let hi = if i + 1 < c_cnt: opt[Col(i + 1), Col(j)] else: c_cnt - 1
            var best = INF32
            var best_k = lo
            for k in lo..=hi:
                let cand = a.d[Col(i), Col(k)] + b.d[Col(k), Col(j)]
                if cand < best:
                    best = cand
                    best_k = k
            out[Col(i), Col(j)] = best
            opt[Col(i), Col(j)] = best_k
    Mat(out)

fn solve() -> List<Int>:
    var out: List<Int> = []
    if r_cnt == 1:
        var mat = build_block(0, 0)
        for ev in events:
            match ev:
                ChangeH(p, q, w) =>
                    h_cost[p, q] = w
                    mat = build_block(0, 0)
                ChangeV(p, q, w) =>
                    v_cost[p, q] = w
                Escape(v1, v2) =>
                    out.push(Int(mat.d[v1, v2]))
        return out

    let n_trans = r_cnt - 1
    let n_blk = (n_trans + BLOCK - 1) / BLOCK
    domain Blk = 0..n_blk

    fn blk_range(b: Blk) -> (Int, Int):
        let lo = b.to_idx * BLOCK
        (lo, min(n_trans - 1, lo + BLOCK - 1))

    // WISH A18: `Monoid<Mat>` requires an identity value (`Mat.identity()`),
    // which wastes O(C^2) memory and forces checking for identity in `combine_monge`
    // (or filling diagonal 0 / off-diagonal INF). When every query only reads
    // `st.all` on a full non-empty domain, a `Semigroup<Mat>` should suffice!
    let id_mat = Mat([Col, Col] (i, j) => if i == j: 0i32 else: INF32)
    let mat_monoid = Monoid<Mat>(id: id_mat, op: (a, b) => combine_monge(a, b))
    var st = SegTree<Blk, Mat>.of(mat_monoid, b => {
        let (lo, hi) = blk_range(b)
        build_block(lo, hi)
    })

    for ev in events:
        match ev:
            ChangeH(p, q, w) =>
                h_cost[p, q] = w
                let b = Blk(min(p.to_idx, n_trans - 1) / BLOCK)
                st.refresh(b)
            ChangeV(p, q, w) =>
                v_cost[p, q] = w
                st.refresh(Blk(p.to_idx / BLOCK))
            Escape(v1, v2) =>
                out.push(Int(st.all.d[v1, v2]))
    out

for ans in solve(): print(ans)
