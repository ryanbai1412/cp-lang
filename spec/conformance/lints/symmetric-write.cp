// expect: L0503
domain V = 1..=3
var adj: [V]List<V> = []
let (u, v) = (V(1), V(2))
adj[u].push(v) // <- here
adj[v].push(u)
print(adj[u].len)
