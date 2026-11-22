// expect: ok, output "YES"
use std.tree.{LinkCut}
domain V = 0..4
let vals: [V]Int = [1, 2, 3, 4]
var lc = LinkCut(+, a: Some(vals))
lc.make_root(V(3))
print(yes(lc.root_of(V(3)) == V(3)))
