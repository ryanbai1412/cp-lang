// expect: ok, output "2 0"
// rule: std.graph.Next
// A table of neighbour lists is a `Next`: `adj.out(u)` walks `adj[u]`.
domain V = 0..3
var adj: [V]List<V> = [V] v => []
adj[V(0)].push(V(1))
adj[V(0)].push(V(2))
print(adj.out(V(0)).count(), adj.out(V(2)).count())
