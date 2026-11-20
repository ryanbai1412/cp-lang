// Runnable today/ port of IOI '13 P3 (Wombats) for real Rust `cpl build`.
// Stdin: r, c, h[r][c-1], v[r-1][c], q, then queries (1/2/3).
// Uses top-level recursive fns (working around cpl-emit recursive nested-fn self-capture bug).
// original: corpus/submissions/4722414.cpp

fn ri() -> Int:
    let x: Int = read()
    return x

const INF = 1_000_000_000_000
const BS = 8
var g_r = 0
var g_c = 0
var g_h: List<List<Int>> = []
var g_v: List<List<Int>> = []
var g_seg: List<List<Int>> = []

fn comb(a: List<Int>, b: List<Int>) -> List<Int>:
    let c = g_c
    var out = [0..c * c] _ => INF
    var opt = [0..c * c] _ => 0
    var i = c - 1
    while i >= 0:
        for j in 0..c:
            let lo = if j > 0: opt[i * c + j - 1] else: 0
            let hi = if i + 1 < c: opt[(i + 1) * c + j] else: c - 1
            var best = INF
            var best_k = lo
            for k in lo..=hi:
                let cand = a[i * c + k] + b[k * c + j]
                if cand < best:
                    best = cand
                    best_k = k
            out[i * c + j] = best
            opt[i * c + j] = best_k
        i -= 1
    return out

fn build_leaf(blk: Int) -> List<Int>:
    let c = g_c
    let lo = blk * BS
    let hi = min(g_r, lo + BS)
    var dp = [0..c * c] _ => INF
    for s in 0..c:
        var cur = [0..c] _ => INF
        cur[s] = 0
        for row in lo..hi:
            for j in 1..c: cur[j] min= cur[j - 1] + g_h[row][j - 1]
            var j = c - 2
            while j >= 0:
                cur[j] min= cur[j + 1] + g_h[row][j]
                j -= 1
            if row + 1 < g_r:
                for j in 0..c: cur[j] += g_v[row][j]
        for j in 0..c: dp[s * c + j] = cur[j]
    return dp

fn seg_build(o: Int, l: Int, rr: Int):
    if l == rr:
        g_seg[o] = build_leaf(l)
        return
    let mid = (l + rr) / 2
    seg_build(o * 2, l, mid)
    seg_build(o * 2 + 1, mid + 1, rr)
    g_seg[o] = comb(g_seg[o * 2], g_seg[o * 2 + 1])

fn seg_update(o: Int, l: Int, rr: Int, blk: Int):
    if l == rr:
        g_seg[o] = build_leaf(l)
        return
    let mid = (l + rr) / 2
    if blk <= mid: seg_update(o * 2, l, mid, blk)
    else: seg_update(o * 2 + 1, mid + 1, rr, blk)
    g_seg[o] = comb(g_seg[o * 2], g_seg[o * 2 + 1])

g_r = ri()
g_c = ri()
for _ in 0..g_r:
    var row: List<Int> = []
    for _ in 0..g_c - 1: row.push(ri())
    g_h.push(row)
for _ in 0..g_r - 1:
    var row: List<Int> = []
    for _ in 0..g_c: row.push(ri())
    g_v.push(row)
let blocks = (g_r + BS - 1) / BS
for _ in 0..4 * blocks + 5: g_seg.push([0..g_c * g_c] _ => INF)
seg_build(1, 0, blocks - 1)

let q = ri()
for _ in 0..q:
    let ty = ri()
    if ty == 1:
        let p = ri()
        let qq = ri()
        let cost = ri()
        g_h[p][qq] = cost
        seg_update(1, 0, blocks - 1, p / BS)
    elif ty == 2:
        let p = ri()
        let qq = ri()
        let cost = ri()
        g_v[p][qq] = cost
        seg_update(1, 0, blocks - 1, p / BS)
    else:
        let v1 = ri()
        let v2 = ri()
        print(g_seg[1][v1 * g_c + v2])
