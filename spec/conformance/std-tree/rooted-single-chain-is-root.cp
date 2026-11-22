// expect: ok, output "1"
use std.tree.{Rooted}
domain V = 0..1
let adj: [V]List<V> = [[]]
let t = Rooted(adj, root: V(0))
print(t.chain(V(0)).count())
