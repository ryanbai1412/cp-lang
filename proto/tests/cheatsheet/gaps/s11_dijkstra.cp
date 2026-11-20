domain V = 1..=3
var adj: [V]List<(V, Int)> = []
adj[V(1)].push((V(2), 5))
let dist = dijkstra(adj, from: V(1))
print(dist)
