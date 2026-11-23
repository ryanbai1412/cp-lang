// Source: IOI 2013 Day 1 (Wombats), standard I/O version, https://dmoj.ca/problem/ioi13p3io
// Problem: an R x C grid of roads (R <= 5000, C <= 200) with wombat counts on the segments;
//   people move east or west on horizontal roads and only south on vertical ones. Up to 500
//   events change one segment's count; up to 200000 events ask the fewest wombats on a route from
//   (0, V1) to (R-1, V2).
// Solution: cut the rows into blocks of 8 and give each block its C x C matrix of least costs
//   from top column to bottom column, built by C row-by-row relaxations in O(8 C^2). A SegTree
//   over blocks composes matrices by min-plus product; the matrices are Monge, so the optimal
//   middle column is monotone and Knuth's bound makes a product O(C^2). A change rebuilds one
//   block and its path, O(C^2 log(R/8)); the root product is kept, so an escape is one lookup.
// original: corpus/submissions/4722412.cpp

let (r_cnt, c_cnt): (Int, Int) = read()
var h_cost: List<List<Int>> = []
for _ in 0..r_cnt:
    var row: List<Int> = []
    for _ in 0..(c_cnt - 1):
        let x: Int = read()
        row.push(x)
    h_cost.push(row)

var v_cost: List<List<Int>> = []
for _ in 0..(r_cnt - 1):
    var row: List<Int> = []
    for _ in 0..c_cnt:
        let x: Int = read()
        row.push(x)
    v_cost.push(row)

let inf: Int = 1000000000
let block = 8

fn relax_row(r: Int, mut dist: List<Int>):
    for c in 1..c_cnt:
        let cand = dist[c - 1] + h_cost[r][c - 1]
        if cand < dist[c]: dist[c] = cand
    var c = c_cnt - 2
    while c >= 0:
        let cand = dist[c + 1] + h_cost[r][c]
        if cand < dist[c]: dist[c] = cand
        c -= 1

fn build_block(r_lo: Int, r_hi: Int) -> List<Int>:
    var d = List.filled(c_cnt * c_cnt, inf)
    let last_trans = r_cnt - 2
    for s in 0..c_cnt:
        var cur = List.filled(c_cnt, inf)
        cur[s] = 0
        for r in r_lo..=r_hi:
            relax_row(r, mut cur)
            if r <= last_trans:
                for j in 0..c_cnt:
                    cur[j] += v_cost[r][j]
        if r_hi == last_trans:
            relax_row(r_cnt - 1, mut cur)
        for j in 0..c_cnt:
            d[s * c_cnt + j] = cur[j]
    d

// Knuth-accelerated Monge min-plus product: O(C^2).
fn combine_monge(a: List<Int>, b: List<Int>) -> List<Int>:
    var out = List.filled(c_cnt * c_cnt, inf)
    var opt = List.filled(c_cnt * c_cnt, 0)
    var i = c_cnt - 1
    while i >= 0:
        for j in 0..c_cnt:
            let lo = if j > 0: opt[i * c_cnt + (j - 1)] else: 0
            let hi = if i + 1 < c_cnt: opt[(i + 1) * c_cnt + j] else: c_cnt - 1
            var best = inf
            var best_k = lo
            for k in lo..=hi:
                let cand = a[i * c_cnt + k] + b[k * c_cnt + j]
                if cand < best:
                    best = cand
                    best_k = k
            out[i * c_cnt + j] = best
            opt[i * c_cnt + j] = best_k
        i -= 1
    out

let e_cnt: Int = read()

if r_cnt == 1:
    var mat = build_block(0, 0)
    for _ in 0..e_cnt:
        let ev: Int = read()
        if ev == 1:
            let (p, q, w): (Int, Int, Int) = read()
            h_cost[p][q] = w
            mat = build_block(0, 0)
        elif ev == 2:
            let (p, q, w): (Int, Int, Int) = read()
            v_cost[p][q] = w
        else:
            let (v1, v2): (Int, Int) = read()
            print(mat[v1 * c_cnt + v2])
else:
    let n_trans = r_cnt - 1
    let n_blk = ceil_div(n_trans, block)
    domain Blk = 0..n_blk

    fn blk_lo(b: Int) -> Int = b * block
    fn blk_hi(b: Int) -> Int = min(n_trans - 1, b * block + block - 1)

    var id_mat = List.filled(c_cnt * c_cnt, inf)
    for i in 0..c_cnt: id_mat[i * c_cnt + i] = 0
    let mat_monoid = Monoid<List<Int>>(id: id_mat, op: (a, b) => combine_monge(a, b))
    let init = [Blk] b => build_block(blk_lo(b.to_idx), blk_hi(b.to_idx))
    var st = SegTree<Blk, List<Int>>(mat_monoid, init)
    // queries outnumber updates 400 to 1, so keep the whole product instead of folding per query
    var top = st.fold(Blk.all)

    for _ in 0..e_cnt:
        let ev: Int = read()
        if ev == 1:
            let (p, q, w): (Int, Int, Int) = read()
            h_cost[p][q] = w
            let b = min(p, n_trans - 1) / block
            st.set(Blk(b), build_block(blk_lo(b), blk_hi(b)))
            top = st.fold(Blk.all)
        elif ev == 2:
            let (p, q, w): (Int, Int, Int) = read()
            v_cost[p][q] = w
            let b = p / block
            st.set(Blk(b), build_block(blk_lo(b), blk_hi(b)))
            top = st.fold(Blk.all)
        else:
            let (v1, v2): (Int, Int) = read()
            print(top[v1 * c_cnt + v2])
