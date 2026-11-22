// expect: ok, output "0 2 2"
// rule: std.tree.Rooted.lca
domain V = 0..3
let adj: [V]List<V> = [[V(1)], [V(0), V(2)], [V(1)]]
let t = Rooted(adj, root: V(0))
print(Int(t.lca(V(0), V(2))), Int(t.lca(V(2), V(2))), t.dist(V(0), V(2)))
