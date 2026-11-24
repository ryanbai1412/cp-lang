// Source: Yet Another Contest 6 P6 (No More Snakes and Ladders), https://dmoj.ca/problem/yac6p6
// Problem: an n x m grid (n, m <= 200) of values <= 1e9. Each of Q <= 1e5 independent queries
//   gives a subgrid (A, B)..(C, D) and K <= 5: a token walks right/down from (A, B) to (C, D)
//   and may first set up to K cells to 0; print the least possible sum of the visited cells.
// Solution: queries with both sides <= 4 run a direct (cell, zeros used) DP. Otherwise the path
//   crosses the anti-diagonal x + y = C + D - ((C + D) mod 3), at most 3 cells from the end:
//   bucket the query at each such pivot. Per pivot with queries, one backward DP over its
//   upper-left region gives the best start-to-pivot cost per zero count, a 3 x 3 forward DP
//   the pivot-to-end cost; a query takes the best split of K. O(nm K) per pivot, O(K^2) a query.
// original: corpus/submissions/5150025.cpp
// Bounds: n, m: 1..=200; q: 1..=200_000
const MAXK = 5
domain KDom = 0..=MAXK
domain D3 = 0..3

let (n, m, q): (1..=200, 1..=200, 1..=200_000) = read()
let a: [0..n, 0..m]Int = read()

type Query = (x1: Int, y1: Int, x2: Int, y2: Int, kv: Int, id: Int)
var qans: [0..q]?Int = None
var buckets: [0..n, 0..m]List<Query> = []

fn small_dp(x1: Int, y1: Int, x2: Int, y2: Int, kv: Int) -> Int:
    let (h, w) = (x2 - x1 + 1, y2 - y1 + 1)
    var dp: [0..h, 0..w, KDom]?Int = None
    for i in (0..h).rev:
        for j in (0..w).rev:
            if i == h - 1 and j == w - 1:
                for k in 0..kv: dp[i, j, KDom(k)] = 0
                dp[i, j, KDom(kv)] = a[x1 + i, y1 + j]
                continue
            let cost = a[x1 + i, y1 + j]
            for k in 0..=kv:
                var best: ?Int = None
                if i + 1 < h:
                    best min= dp[i + 1, j, KDom(k)] + cost
                    if k + 1 <= kv: best min= dp[i + 1, j, KDom(k + 1)]
                if j + 1 < w:
                    best min= dp[i, j + 1, KDom(k)] + cost
                    if k + 1 <= kv: best min= dp[i, j + 1, KDom(k + 1)]
                dp[i, j, KDom(k)] = best
    dp[0, 0, KDom(0)]!

for id in 0..q:
    let (x1i, y1i, x2i, y2i, kv): (1..=200, 1..=200, 1..=200, 1..=200, 0..=MAXK) = read()
    let (x1, y1, x2, y2) = (x1i - 1, y1i - 1, x2i - 1, y2i - 1)
    if x2 - x1 <= 3 and y2 - y1 <= 3:
        qans[id] = small_dp(x1, y1, x2, y2, kv)
        continue
    let rem = (x2 + y2) % 3
    for dx in 0..=rem:
        let (cx, cy) = (x2 - dx, y2 - (rem - dx))
        if cx >= x1 and cy >= y1:
            buckets[cx, cy].push((x1:, y1:, x2:, y2:, kv:, id:))

var dp: [0..n, 0..m, KDom]?Int = None
var dp2: [D3, D3, KDom]?Int = None

for cx in 0..n:
    for cy in 0..m:
        if buckets[cx, cy].is_empty: continue
        for dr in 0..3:
            for dc in 0..3:
                let (r, c) = (cx + dr, cy + dc)
                if r >= n or c >= m: continue
                if dr == 0 and dc == 0:
                    for k in 0..=MAXK: dp2[D3(0), D3(0), KDom(k)] = 0
                    continue
                for k in 0..=MAXK:
                    var best: ?Int = None
                    if dr > 0:
                        best min= dp2[D3(dr - 1), D3(dc), KDom(k)] + a[r, c]
                        if k > 0: best min= dp2[D3(dr - 1), D3(dc), KDom(k - 1)]
                    if dc > 0:
                        best min= dp2[D3(dr), D3(dc - 1), KDom(k)] + a[r, c]
                        if k > 0: best min= dp2[D3(dr), D3(dc - 1), KDom(k - 1)]
                    dp2[D3(dr), D3(dc), KDom(k)] = best

        for r in (0..=cx).rev:
            for c in (0..=cy).rev:
                if r == cx and c == cy:
                    dp[r, c, KDom(0)] = a[r, c]
                    for k in 1..=MAXK: dp[r, c, KDom(k)] = 0
                    continue
                let cost = a[r, c]
                for k in 0..=MAXK:
                    var best: ?Int = None
                    if r + 1 <= cx:
                        best min= dp[r + 1, c, KDom(k)] + cost
                        if k > 0: best min= dp[r + 1, c, KDom(k - 1)]
                    if c + 1 <= cy:
                        best min= dp[r, c + 1, KDom(k)] + cost
                        if k > 0: best min= dp[r, c + 1, KDom(k - 1)]
                    dp[r, c, KDom(k)] = best

        for qr in buckets[cx, cy]:
            let (dr, dc) = (qr.x2 - cx, qr.y2 - cy)
            for k in 0..=qr.kv:
                qans[qr.id] min= dp[qr.x1, qr.y1, KDom(k)] + dp2[D3(dr), D3(dc), KDom(qr.kv - k)]

for id in 0..q: print(qans[id]!)
