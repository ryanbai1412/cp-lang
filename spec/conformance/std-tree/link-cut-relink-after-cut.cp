// expect: ok, output "YES YES"
use std.tree.{LinkCut}
domain V = 0..3
var lc = LinkCut<V, Int>(+)
lc.link(V(0), V(1))
lc.cut(V(0), V(1))
let apart = not lc.connected(V(0), V(1))
lc.link(V(1), V(0))
print(yes(apart), yes(lc.connected(V(0), V(1))))
