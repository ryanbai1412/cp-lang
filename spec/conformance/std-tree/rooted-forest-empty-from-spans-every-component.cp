// expect: ok, output "3 0 2 3"
// rule: std.tree.Rooted.forest
// With no roots given, every component becomes a tree, rooted at its first vertex in `V` order (decision 450).
domain V = 0..5
let adj: [V]List<V> = [[V(2)], [], [V(0)], [V(4)], [V(3)]]
let from: List<V> = []
let t = Rooted.forest(adj, from: from)
print(t.roots.count(), Int(t.root_of(V(2))), Int(t.root_of(V(4))) - 1, Int(t.root_of(V(3))))
