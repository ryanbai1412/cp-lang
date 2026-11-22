// expect: ok, output "0 YES"
use std.tree.{Rooted}
domain V = 0..1
let adj: [V]List<V> = [[]]
let t = Rooted(adj, root: V(0))
let lp = t.long_paths()
print(lp.height[V(0)], yes(lp.deep[V(0)] == None))
