// CTSC '04 P4: minimum-cost selection of at most m hubs in a weighted tree.
// Rooted<V> provides subtree sizes and children; all-pairs tree distances feed a
// bottom-up compressed monotone knapsack DP with Tropical Optional (?Int) transitions.
// original: corpus/submissions/4529641.cpp

let (n, raw_m): (Int, Int) = read()
let m = min(raw_m - 1, n)
domain V = 0..n
let cost: [V]Int = read()

var edges: List<(V, V, Int)> = []
for _ in 0..n - 1:
    let (u, v, w): (1..=n, 1..=n, Int) = read()
    edges.push((V(u - 1), V(v - 1), w))

let g = Graph.undirected(edges)
let t = Rooted(g, V.first)

var dist: [V, V]Int = 0
for s in V:
    var d: [V]?Int = none
    d[s] = 0
    var stk: List<V> = [s]
    while let u = stk.pop():
        for (v, w) in g[u]:
            if d[v] == none:
                d[v] = d[u]! + w
                stk.push(v)
    for v in V: dist[s, v] = d[v]!

fn compress_dp(f: [0..=m]?Int) -> List<(Int, Int)>:
    var out: List<(Int, Int)> = []
    var prev: ?Int = none
    for i in 0..=m:
        if let val = f[i]:
            if prev == none or val != prev!:
                out.push((i, val))
                prev = val
    out

fn dp(u: V) -> [V]List<(Int, Int)>:
    let kids = sorted(t.children(u), by: v => -t.size(v))
    let child_dp = [dp(v) for v in kids]

    var raw: [V]List<(Int, Int)> = []
    for v in V:
        var f: [0..=m]?Int = none
        f[0] = dist[u, v]
        var csz = 1
        for ci in 0..kids.len:
            for j in (0..=min(m, csz)).rev:
                let cur = f[j]
                f[j] = none
                for (x, y) in child_dp[ci][v]:
                    if j + x > m: break
                    f[j + x] min= cur + y
            csz += t.size(kids[ci])
        raw[v] = compress_dp(f)

    var out: [V]List<(Int, Int)> = []
    var cur: [0..=m]?Int = none
    for a in sorted(V, by: v => dist[u, v]):
        var acur: [0..=m]?Int = none
        var j = 0
        var best: ?Int = none
        for i in 0..=m:
            while j < raw[a].len and raw[a][j].0 <= i:
                best min= raw[a][j].1
                j += 1
            acur[i] = best
        let merged: [0..=m]?Int = [0..=m] i => min(cur[i], acur[i])
        out[a] = compress_dp(merged)
        for i in 1..=m: cur[i] min= acur[i - 1] + cost[a]
    out

let res = dp(V.first)
var best_cost: ?Int = none
var best_cnt = 0
for v in V:
    let (up, c) = res[v].last!
    let tot = c + cost[v]
    if best_cost == none or tot < best_cost! or (tot == best_cost! and up + 1 < best_cnt):
        best_cost = tot
        best_cnt = up + 1

print(best_cost!)
print(best_cnt)
