// Source: Codeforces 2057E2 (Another Exercise on Graphs, hard version),
//   https://codeforces.com/contest/2057/problem/E2
// Problem: a connected weighted graph, n <= 400, no multi-edges, q <= 3e5 queries (a, b, k):
//   over all a-b paths, the smallest k-th largest edge weight (every path has >= k edges).
//   Multitest, sums of n <= 400 and q <= 3e5.
// Solution: the answer is <= x iff with edges of weight <= x costing 0 and the rest 1,
//   dist(a, b) < k. Start with all edges at 1 (Floyd-Warshall), then zero edges by increasing
//   weight; only an edge joining two zero-cost components changes dist (at most n - 1 times),
//   by an O(n^2) relaxation through it. Queries wait per pair sorted by k and are answered by
//   the weight that brings dist below k. O(n^3 + q log q).
// Bounds: n: 2..=400; q: 1..=300_000; queries.a, queries.b: 1..=n
@!multitest
use std.graph.{floyd_warshall}
let (n, m, q): (2..=400, Int, 1..=300_000) = read()
domain V = 1..=n
domain Q = 0..q
let edges = read<[m](u: V, v: V, w: 1..=1e9)>().sorted(by: e => e.w)
let queries: [Q](a: V, b: V, k: Int) = read()

var hop: [V, V]?Int = None
for e in edges:
    for (x, y) in [(e.u, e.v), (e.v, e.u)]: hop[x, y] = 1
assert floyd_warshall(mut hop)
var d = [V, V] (i, j) => hop[i, j]!

var pending: [V, V]List<Q> = []
for i in Q.sort(by: i => queries[i].k): pending[queries[i].a, queries[i].b].push(i)

var ans: [Q]Int = 0
for e in edges:
    if d[e.u, e.v] > 0:
        for i in V:
            for j in V:
                let via = min(d[i, e.u] + d[e.v, j], d[i, e.v] + d[e.u, j])
                if via < d[i, j]:
                    d[i, j] = via
                    while let x = pending[i, j].last, queries[x].k > via:
                        ans[x] = e.w
                        pending[i, j].pop()
print(ans)
