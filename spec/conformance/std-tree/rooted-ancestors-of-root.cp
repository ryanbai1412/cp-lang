// expect: ok, output "1 4"
use std.tree.{Rooted}
domain V = 0..4
let adj: [V]List<V> = [[V(1)], [V(0), V(2)], [V(1), V(3)], [V(2)]]
let t = Rooted(adj, root: V(0))
print(t.ancestors(V(0)).count(), t.ancestors(V(3)).count())
