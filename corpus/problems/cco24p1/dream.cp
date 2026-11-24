// CCO 2024 P1 Treasure Hunt: for each city u, the best value of v[w] - dist(u, w) over all w.
// Negate and it is one multi-source Dijkstra, each source w starting at -v[w].
let (n, m): (1..=1e6, 0..=1e6) = read()
domain V = 1..=n
let value: [V]0..=1e9 = read()
let g = Graph.undirected(read<[m](u: V, v: V, w: 0..=1e9)>())
let d = dijkstra(g, from: V.map(w => (w, -value[w])))
for u in V: print(-d[u]!)
