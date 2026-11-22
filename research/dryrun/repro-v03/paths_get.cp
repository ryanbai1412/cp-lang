// `Paths.get(v)`, the method behind `p[v]`, is rejected by `cpl run` as well (see paths_index.cp).
domain V = 1..=3
let g = Graph.directed([(u: V(1), v: V(2), w: 5), (u: V(2), v: V(3), w: 7)])
let p = dijkstra(g, from: [(V(1), 0)])
print(p.get(V(3))!)
