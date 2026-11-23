// expect: ok, output "0 0"
domain V = 0..3
let adj: [V]List<V> = [[V(1)], [V(0), V(2)], [V(1)]]
let t = Rooted(adj, root: V(0))
print(t.light(V(0)).count(), t.light(V(1)).count())
