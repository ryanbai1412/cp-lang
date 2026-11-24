// Source: Codeforces 2101E (Kia Bakes a Cake), https://codeforces.com/contest/2101/problem/E
// limits: 6000 ms, 512 MB
// Problem: a tree on n <= 7e4 vertices and a binary string s. Over the vertices with s_i = 1,
//   with tree distance as weight, a path is nice if each edge weighs at least twice the previous.
//   For each i print the most vertices on a nice simple path starting at i, or -1 if s_i = 0.
//   Multitest, sum n <= 7e4.
// Solution: build the paths backwards, one round per extra vertex (O(log n) rounds as weights
//   double). max_budget[w]: the largest first step a walk of r - 1 vertices from w allows; u
//   extends via w when dist(u, w) <= max_budget[w], and its new budget is the best such distance
//   halved. Each round, in every centroid piece bucket w at max_budget[w] - depth(w), keep
//   suffix maxima of the deepest w in two different branches, and query each u at its depth.
//   O(n log^2 n).
// Bounds: n: 1..=70_000; edges.u, edges.v: 1..=n

use std.tree.Centroids

type Far<V> = (depth: Int, branch: V)

struct Best<V>:
    far: ?Far<V>
    other: ?Int

    fn best_other(self, br: V) -> ?Int:
        let top = far ?? return None
        if top.branch == br: other else: Some(top.depth)

fn merge<V>(a: Best<V>, o: Best<V>) -> Best<V>:
    let x = a.far ?? return o
    let y = o.far ?? return a
    if x.branch == y.branch: Best((max(x.depth, y.depth), x.branch), max(a.other, o.other))
    elif y.depth > x.depth: Best(y, max(Some(x.depth), o.other))
    else: Best(x, max(a.other, Some(y.depth)))

@!multitest
let n: 1..=70_000 = read()
domain V = 1..=n
let s: [V]Char = read()
let edges: [0..n - 1](u: V, v: V) = read()
let adj = Graph.undirected(edges)
let cd = Centroids<V>.new(adj)

var ans: [V]Int = [V] v => if s[v] == '1': 1 else: -1
var max_budget: [V]Int = [V] v => if s[v] == '1': n else: 0
for round in 2..:
    var next_dist: [V]Int = 0
    for c in V:
        let piece = cd.piece(c)
        let max_d = [d for (_, d, _) in piece].max() ?? 0
        var bests = List.filled(max_d + 1, Best<V>(None, None))
        for (u, d, br) in piece if max_budget[u] > 0 and max_budget[u] >= d:
            let idx = min(max_d, max_budget[u] - d)
            bests[idx] = merge(bests[idx], Best((d, br), None))
        for i in (0..bests.len - 1).rev: bests[i] = merge(bests[i], bests[i + 1])
        for (u, d, br) in piece if s[u] == '1':
            if let far = bests[d].best_other(br): next_dist[u] max= d + far

    let reached = [v for v in V if next_dist[v] > 0]
    if reached.is_empty: break
    for v in reached: ans[v] = round
    max_budget = [V] v => next_dist[v] / 2
print(ans)
