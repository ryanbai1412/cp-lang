let (n, m): (1..=200000, 1..=200000) = read()
domain V = 1..=n
domain E = 0..m
let es: [E](u: V, v: V, w: 1..=1000000000) = read()
let p = dijkstra(Graph.directed(es), from: [(V(1), 0)])
print(p[V(n)] ?? -1)
