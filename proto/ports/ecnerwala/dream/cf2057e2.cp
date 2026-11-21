// Codeforces 2057 E2: the smallest k-th largest edge weight over the paths from a to b.
// The answer is at most x iff some path has fewer than k edges heavier than x: with edges of
// weight <= x costing 0 and the rest 1, dist(a, b) < k. Lighten the edges by increasing weight;
// only an edge joining two zero-cost components changes dist, by one O(n^2) relaxation through it.
@!multitest
let (n, m, q) = read<(2..=400, Int, 1..=3e5)>()
domain V = 1..=n
domain Q = 0..q
let edges = read<[m](u: V, v: V, w: 1..=1e9)>().sorted(by: .w)
let queries: [Q](a: V, b: V, k: Int) = read()

var d = Graph<V>.undirected(edges).hops()
var pending = Q.sorted(by: .k).group(by: i => (queries[i].a, queries[i].b))
var ans: [Q]Int
for e in edges if d[e.u, e.v] > 0:
    for (i, j) in V * V:
        let via = min(d[i, e.u] + d[e.v, j], d[i, e.v] + d[e.u, j])
        if via < d[i, j]:
            d[i, j] = via
            for x in pending[i, j].pop_while(x => queries[x].k > via): ans[x] = e.w
print(ans)
