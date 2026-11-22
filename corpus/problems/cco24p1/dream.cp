# CCO 2024 P1 Treasure Hunt: for each city u, the best value of v[w] - dist(u, w) over all w.
# Negate and it is one multi-source Dijkstra, each source w starting at -v[w].
domain V = [read Int]
let m = read Int
let value = read [V]Int
let g = Graph<V>.undirected(read [m] (u: V, v: V, w: Int))
let d = dijkstra(g, from: V.map(w => (w, -value[w])))
for u in V: print(-d[u]!)
