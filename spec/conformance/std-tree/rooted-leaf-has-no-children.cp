// expect: ok, output "0 1"
use std.tree.{Rooted}
domain V = 0..3
let adj: [V]List<V> = [[V(1), V(2)], [V(0)], [V(0)]]
let t = Rooted(adj, root: V(0))
print(t.children(V(2)).count(), t.subtree(V(2)).count())
