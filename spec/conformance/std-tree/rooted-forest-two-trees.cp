// expect: ok, output "2 4 0 2"
use std.tree.{Rooted}
domain V = 0..4
let adj: [V]List<V> = [[V(1)], [V(0)], [V(3)], [V(2)]]
let t = Rooted.forest(adj, from: [V(0), V(2)])
print(t.roots.count(), t.subtree(V(0)).count() + t.subtree(V(2)).count(), t.depth(V(2)), t.size(V(2)))
