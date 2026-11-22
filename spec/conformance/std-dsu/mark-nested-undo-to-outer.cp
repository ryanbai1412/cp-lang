// expect: ok, output "3"
use std.dsu.{Dsu, Mark}
domain V = 0..3
var d = Dsu<V>()
let outer: Mark = d.mark()
d.unite(V(0), V(1))
let inner: Mark = d.mark()
d.unite(V(1), V(2))
d.undo_to(outer)
print(d.components)
