// expect: ok, output "0 YES"
use std.tree.{Rooted}
domain V = 0..1
var adj: [V]List<V> = []
let t = Rooted(adj, root: V(0))
let d = t.diameter()
print(d.len, yes(d.u == d.v))
