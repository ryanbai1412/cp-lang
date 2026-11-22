// CSES 1671 Shortest Routes I: distances from city 1 over directed weighted flights.
let (n, m): (2..=1e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let flights: [E](u: V, v: V, c: 1..=1e9) = read()
let g = Graph.directed(flights)
let dist = dijkstra(g, from: [(V.first, 0)], w: a => flights[a.e].c)
print([dist[v]! for v in V])
