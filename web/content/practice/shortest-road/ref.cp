let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
let es: [0..m](u: V, v: V, w: 1..=1e9) = read()
let p = dijkstra(Graph.undirected(es), from: [(V(1), 0)])
print(p[V(n)] ?? -1)
