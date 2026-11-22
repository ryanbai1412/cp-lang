// expect: ok, output "1 0"
use std.tree.{Rooted}
domain V = 0..1
var adj: [V]List<V> = []
let t = Rooted(adj, root: V(0))
print(t.subtree_sizes[V(0)], Int(t.centroid()))
