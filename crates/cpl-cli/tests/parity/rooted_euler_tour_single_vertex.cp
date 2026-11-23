domain V = 0..1
let adj: [V]List<V> = [[]]
let t = Rooted(adj, root: V(0))
print(yes(t.euler_tour().collect() == [V(0)]))
