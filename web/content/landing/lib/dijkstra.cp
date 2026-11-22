let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let roads: [E](u: V, v: V, w: 1..=1e9) = read()
let g = Graph.directed(roads)
let d = dijkstra(g, from: [(V(1), 0)], w: a => roads[a.e].w)
for v in V:
    print(d[v] ?? -1)
