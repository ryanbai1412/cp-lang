let (n, m) = read<(2..=1e5, 1..=2e5)>()
domain V = 1..=n
let flights = read<[m](u: V, v: V, w: 1..=1e9)>()
let g = Graph.directed(flights)
print(dijkstra(g, from: [(V(1), 0)]).dist, none: -1)
