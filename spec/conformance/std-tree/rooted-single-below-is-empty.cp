// expect: ok, output "0"
domain V = 0..1
let adj: [V]List<V> = [[]]
let t = Rooted(adj, root: V(0))
print(t.below(V(0)).count())
