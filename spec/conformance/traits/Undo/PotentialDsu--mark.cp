// expect: ok, output "YES NO"
use std.dsu.{PotentialDsu}
domain V = 0..3
var d = PotentialDsu<V>()
d.unite(V(0), V(1), 5)
let m = d.mark()
d.unite(V(1), V(2), 5)
d.undo_to(m)
print(yes(d.same(V(0), V(1))), yes(d.same(V(1), V(2))))
