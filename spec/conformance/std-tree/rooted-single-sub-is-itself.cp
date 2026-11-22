// expect: ok, output "YES"
use std.tree.{Rooted}
domain V = 0..1
let adj: [V]List<V> = [[]]
let t = Rooted(adj, root: V(0))
print(yes(t.sub(V(0)).map(p => t.at[p]).collect() == [V(0)]))
