let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
let g = Graph.directed(read<[m](u: V, v: V, w: 1..=1e9)>())
let d = dijkstra(g, from: [(V(1), 0)])
for v in V:
    print(d.dist[v] ?? -1)
