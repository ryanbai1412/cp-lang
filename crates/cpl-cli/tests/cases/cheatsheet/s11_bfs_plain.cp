domain V = 1..=4
var adj: [V]List<V> = []
adj[V(1)].push(V(2))
let dist = bfs(from: [V(1)], next: u => adj[u])
print(dist)
