// expect: R0401 (checked)
// rule: std.tree.Rooted.lower
domain V = 0..3
let adj: [V]List<V> = [[V(1)], [V(0), V(2)], [V(1)]]
let t = Rooted(adj, root: V(0))
print(Int(t.lower(V(0), V(2)))) // <- here
