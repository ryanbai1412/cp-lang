// expect: R0401 (checked)
// rule: std.tree.Rooted.orient
// Two vertices on different branches are not an ancestor pair, so there is no edge of the DFS tree's graph to orient.
domain V = 0..3
let adj: [V]List<V> = [[V(1), V(2)], [V(0)], [V(0)]]
let t = Rooted(adj, root: V(0))
print(Int(t.orient(V(1), V(2)).anc)) // <- here
