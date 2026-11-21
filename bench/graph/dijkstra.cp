// Dijkstra from vertex 1 over an undirected graph; prints the sum of the
// reachable distances.
let (n, m): (1..=1e6, 0..=2e6) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V, w: 1..=1e9) = read()
let dist = dijkstra(Graph.undirected(edges), from: [(V(1), 0)], w: a => edges[a.e].w)
var total = 0
for v in V:
    if let d = dist[v]: total += d
print(total)
