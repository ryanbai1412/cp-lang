// expect: ok, output "NO YES"
use std.dsu.{PotentialDsu}
domain V = 0..3
var d = PotentialDsu<V>()
let m = d.mark()
d.unite(V(0), V(2), 5)
d.undo_to(m)
d.unite(V(1), V(2), 5)
print(yes(d.same(V(0), V(2))), yes(d.same(V(1), V(2))))
