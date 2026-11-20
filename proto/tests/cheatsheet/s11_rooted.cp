domain V = 1..=3
var adj: [V]List<V> = []
adj[V(1)].push(V(2)); adj[V(2)].push(V(1)); adj[V(2)].push(V(3)); adj[V(3)].push(V(2))
let t = Rooted(adj, root: V(1))
print(t.parent(V(3)), t.depth(V(3)), t.size(V(1)), t.lca(V(2), V(3)), t.dist(V(1), V(3)), t.is_anc(V(1), V(3)))
