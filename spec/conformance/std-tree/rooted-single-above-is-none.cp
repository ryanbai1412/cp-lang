// expect: ok, output "YES"
use std.tree.{Rooted}
domain V = 0..1
let adj: [V]List<V> = [[]]
let t = Rooted(adj, root: V(0))
print(yes(t.above(+, finish: (s, v) => s + 1) == [None]))
