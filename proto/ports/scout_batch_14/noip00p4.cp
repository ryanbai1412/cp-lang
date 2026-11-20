// NOIP 2000 Square Number Picking: send two down/right paths across an n x n
// grid to collect the largest weight sum (each cell counted once). Solved by
// enumerating path subsets on the grid and combining disjoint halves via SOS DP.
// original: corpus/submissions/5067920.cpp
use std.algebra.{Any, Max}                                        // WISH P10
use std.subset.{sos_sub, sos_sup}                                 // WISH P10

let n: 1..=9 = read()
var cell_id: [0..n, 0..n]Int = -1
var weights: List<Int> = []
while true:                                                       // WISH P3
    let (x, y, z): (0..=9, 0..=9, 0..=1000) = read()
    if (x, y, z) == (0, 0, 0): break
    cell_id[x - 1, y - 1] = weights.len
    weights.push(z)

let m: 0..=20 = weights.len
domain Item = 0..m
type Mask = Subset<Item>

// dp[r, c]: distinct subsets of weighted cells reachable on a path from (r, c) to (n-1, n-1).
var dp: [0..n, 0..n]List<Mask> = []
for r in (0..n).rev:
    for c in (0..n).rev:
        var cur: List<Mask> = []
        if r == n - 1 and c == n - 1: cur.push(Mask.empty)
        if r + 1 < n: cur.extend(dp[r + 1, c])
        if c + 1 < n: cur.extend(dp[r, c + 1])
        var uniq = sorted(cur, by: s => s.idx).dedup()            // WISH P14
        let id = cell_id[r, c]
        if id != -1:
            let item = Item(id)
            for i in 0..uniq.len: uniq[i] = uniq[i].with(item)
        dp[r, c] = uniq

var reach: [Mask]Bool = false
for s in dp[0, 0]: reach[s] = true

var total: [Mask]Int = 0
var best_sub: [Mask]Int = 0
for s in Mask:
    let w = sum(weights[i.idx] for i in s)
    total[s] = w
    if reach[s]: best_sub[s] = w

sos_sup(Any, reach)                                               // WISH P10
sos_sub(Max, best_sub)                                            // WISH P10

print(max(total[s] + best_sub[Mask.full - s] for s in Mask if reach[s]))
