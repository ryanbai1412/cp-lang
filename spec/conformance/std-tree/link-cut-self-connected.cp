// expect: ok, output "YES"
use std.tree.{LinkCut}
domain V = 0..2
var lc = LinkCut<V, Int>(+)
print(yes(lc.connected(V(1), V(1))))
