// CSES 1672 Shortest Routes II: all-pairs shortest paths on an undirected graph.
// Input: n m q, m roads "a b c", q queries "a b". Output: per query the distance, or -1.
domain V = 1..=read<1..=500>()
let m = read<1..=250000>()
let q = read<1..=1e5>()
let roads = read<[m](u: V, v: V, w: 1..=1e9)>()
let g = Graph.undirected(roads, keep: r => (to: r.to, w: Int(r.w)))
var d = g.matrix()
let ok = floyd_warshall(mut d)
assert ok
for (u:, v:) in read<[q](u: V, v: V)>():
    print(d[u, v] ?? -1)
