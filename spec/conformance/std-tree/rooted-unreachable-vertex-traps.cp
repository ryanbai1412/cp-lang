// expect: R0401 (checked)
// rule: std.tree.Rooted.new
domain V = 0..3
let adj: [V]List<V> = [[V(1)], [V(0)], []]
let t = Rooted(adj, root: V(0)) // <- here
print(t.dist(V(0), V(1)))
