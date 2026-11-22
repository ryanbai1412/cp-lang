// expect: R0401 at 6 (checked)
use std.tree.{LinkCut}
domain V = 0..3
var lc = LinkCut<V, Int>(+)
lc.link(V(0), V(1))
lc.link(V(1), V(2))
lc.cut(V(0), V(2))
