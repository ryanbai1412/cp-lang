// expect: ok, output "YES"
use std.tree.{Rooted}
domain V = 0..1
let adj: [V]List<V> = [[]]
let t = Rooted(adj, root: V(0))
print(yes(t.preorder(V(0)).map(s => (s.state, s.depth)).collect() == [(V(0), 0)]))
