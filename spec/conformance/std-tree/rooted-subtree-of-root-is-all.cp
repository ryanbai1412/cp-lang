// expect: ok, output "3"
domain V = 0..3
let adj: [V]List<V> = [[V(1)], [V(0), V(2)], [V(1)]]
let t = Rooted(adj, root: V(0))
print(t.subtree(V(0)).count())
