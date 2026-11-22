// CSES 1671 Shortest Routes I: distances from city 1 over directed weighted flights.
let (n, m): (2..=1e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let flights = read<[E](u: V, v: V, w: 1..=1e9)>()
let p = dijkstra(Graph.directed(flights), from: [(V.first!, 0)])
print([p[v]! for v in V])
