// expect: ok, output "YES"
domain V = 0..1
let adj: [V]List<V> = [[]]
let t = Rooted(adj, root: V(0))
print(yes(t.on_path(V(0), V(0), V(0))))
