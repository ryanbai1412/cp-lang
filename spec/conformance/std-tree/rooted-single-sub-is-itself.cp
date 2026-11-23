// expect: ok, output "YES"
domain V = 0..1
let adj: [V]List<V> = [[]]
let t = Rooted(adj, root: V(0))
print(yes(t.sub(V(0)).map(p => t.at[p]).collect() == [V(0)]))
