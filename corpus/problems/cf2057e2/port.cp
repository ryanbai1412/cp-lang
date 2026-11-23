// Codeforces 2057 E2: the smallest k-th largest edge weight over the paths from a to b.
// The answer is at most x iff some path has fewer than k edges heavier than x: with edges of
// weight <= x costing 0 and the rest 1, dist(a, b) < k. Lighten the edges by increasing weight;
// only an edge joining two zero-cost components changes dist, by one O(n^2) relaxation through it.
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
