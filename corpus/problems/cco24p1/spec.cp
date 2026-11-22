// cco24p1 Treasure Hunt: from each cave, the best a[u] - dist(v, u) over every cave u. Negated,
// that is a shortest path from every cave at once, cave u starting at -a[u].
let (n, m) = read<(1..=200_000, 0..=200_000)>()
domain V = 1..=n
let a = read<[V]1..=1_000_000_000>()
let g = Graph.undirected(read<[m](u: V, v: V, w: 1..=1_000_000_000)>())
let p = dijkstra(g, from: V.map(v => (v, -a[v])))
for v in V:
    print(-p[v]!)
