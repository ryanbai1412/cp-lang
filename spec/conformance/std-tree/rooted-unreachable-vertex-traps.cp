// expect: R0401 at 3 (checked)
// rule: std.tree.Rooted.new
domain V = 0..3
let adj: [V]List<V> = [[V(1)], [V(0)], []]
let t = Rooted(adj, root: V(0))
print(t.dist(V(0), V(1)))
