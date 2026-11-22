// `Paths.get` is `p[v]`. `cpl check` accepts; `cpl run` rejects the index.
domain V = 1..=3
let g = Graph.directed([(u: V(1), v: V(2), w: 5), (u: V(2), v: V(3), w: 7)])
let p = dijkstra(g, from: [(V(1), 0)])
print(p[V(3)]!)
