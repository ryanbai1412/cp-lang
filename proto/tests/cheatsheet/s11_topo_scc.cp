domain V = 1..=3
var adj: [V]List<V> = []
adj[V(1)].push(V(2))
print(topo(adj), scc(adj).len)
