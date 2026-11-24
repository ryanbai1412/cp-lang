// expect: ok, output "2 0 0 3"
// rule: std.tree.Rooted.forest
// A root an earlier root already reached is skipped, as `dfs` skips a seen start, and a vertex no root reaches roots its own tree (decision 451).
domain V = 0..4
let adj: [V]List<V> = [[V(1)], [V(0), V(2)], [V(1)], []]
let t = Rooted.forest(adj, from: [V(0), V(2)])
print(t.roots.count(), Int(t.root_of(V(2))), t.depth(V(0)), Int(t.root_of(V(3))))
