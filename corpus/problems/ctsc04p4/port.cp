// Source: CTSC 2004 (Network Upgrade), https://dmoj.ca/problem/ctsc04p4
// Problem: a tree of n <= 400 switches with edge lengths < 20000 and upgrade costs c_i. Upgrade
//   1..p switches to cores; cost is the sum of their c_i plus, for every other switch, the tree
//   distance to its nearest core. Print the least cost, then the number of cores used.
// Solution: all-pairs distances by n tree walks. dp(u)[v] is a list (cores opened in u's
//   subtree, least cost) for that subtree when u's nearest core is at most dist(u, v) away and v
//   is open for free: children merge knapsack-style capped by subtree size; then, scanning v
//   by distance from u, either u uses v or a closer core a opened at c_a. Lists keep only points
//   where the cost drops. Answer: min over v of dp(root)[v] with <= p - 1 cores, plus c_v.
//   About O(n^2 p).
// Uses top-level recursive fns (working around cpl-emit recursive nested-fn self-capture bug).
// original: corpus/submissions/4529641.cpp

fn ri() -> Int:
    let x: Int = read()
    return x

const INF = 1_000_000_000_000_000
var g_n = 0
var g_m = 0
var g_cost: List<Int> = []
var g_adj: List<List<(Int, Int)>> = []
var g_dist: List<Int> = []
var g_kids: List<List<Int>> = []
var g_sz: List<Int> = []

fn build_tree(u: Int, p: Int):
    g_sz[u] = 1
    for e in g_adj[u]:
        let v = e.0
        if v != p:
            g_kids[u].push(v)
            build_tree(v, u)
            g_sz[u] += g_sz[v]

fn compress_dp(f: List<Int>) -> List<(Int, Int)>:
    var out: List<(Int, Int)> = []
    var prev = INF
    for i in 0..=g_m:
        if f[i] < INF and f[i] != prev:
            out.push((i, f[i]))
            prev = f[i]
    return out

fn dp(u: Int) -> List<List<(Int, Int)>>:
    var ord_kids: List<(Int, Int)> = [(-g_sz[v], v) for v in g_kids[u]]
    ord_kids.sort()
    var child_dp: List<List<List<(Int, Int)>>> = [dp(p.1) for p in ord_kids]

    var raw: List<List<(Int, Int)>> = []
    for v in 0..g_n:
        var f = [0..g_m + 1] _ => INF
        f[0] = g_dist[u * g_n + v]
        var csz = 1
        for ci in 0..ord_kids.len:
            var j = min(g_m, csz)
            while j >= 0:
                let cur = f[j]
                f[j] = INF
                if cur < INF:
                    for pair in child_dp[ci][v]:
                        if j + pair.0 > g_m: break
                        f[j + pair.0] min= cur + pair.1
                j -= 1
            csz += g_sz[ord_kids[ci].1]
        raw.push(compress_dp(f))

    var ord_v: List<(Int, Int)> = [(g_dist[u * g_n + v], v) for v in 0..g_n]
    ord_v.sort()

    var out: List<List<(Int, Int)>> = [[] for _ in 0..g_n]
    var cur = [0..g_m + 1] _ => INF
    for pair in ord_v:
        let a = pair.1
        var acur = [0..g_m + 1] _ => INF
        var j = 0
        var best = INF
        for i in 0..=g_m:
            while j < raw[a].len and raw[a][j].0 <= i:
                best min= raw[a][j].1
                j += 1
            acur[i] = best
        var merged = [0..g_m + 1] _ => INF
        for i in 0..=g_m: merged[i] = min(cur[i], acur[i])
        out[a] = compress_dp(merged)
        for i in 1..=g_m:
            if acur[i - 1] < INF:
                cur[i] min= acur[i - 1] + g_cost[a]
    return out

g_n = ri()
let raw_m = ri()
g_m = min(raw_m - 1, g_n)
for _ in 0..g_n: g_cost.push(ri())
for _ in 0..g_n:
    g_adj.push([])
    g_kids.push([])
for _ in 0..g_n - 1:
    let u = ri() - 1
    let v = ri() - 1
    let w = ri()
    g_adj[u].push((v, w))
    g_adj[v].push((u, w))

g_dist = [0..g_n * g_n] _ => 0
for s in 0..g_n:
    var d = [0..g_n] _ => -1
    d[s] = 0
    var stk: List<Int> = [s]
    while stk.len > 0:
        let u = stk.pop()!
        for e in g_adj[u]:
            if d[e.0] == -1:
                d[e.0] = d[u] + e.1
                stk.push(e.0)
    for v in 0..g_n: g_dist[s * g_n + v] = d[v]

g_sz = [0..g_n] _ => 0
build_tree(0, -1)
let res = dp(0)
var best_cost = INF
var best_cnt = 0
for v in 0..g_n:
    let last = res[v][res[v].len - 1]
    let tot = last.1 + g_cost[v]
    if tot < best_cost or (tot == best_cost and last.0 + 1 < best_cnt):
        best_cost = tot
        best_cnt = last.0 + 1
print(best_cost)
print(best_cnt)
