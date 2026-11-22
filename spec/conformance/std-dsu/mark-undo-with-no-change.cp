// expect: ok, output "3 YES"
use std.dsu.{Dsu, Mark}
domain V = 0..3
var d = Dsu<V>()
let m: Mark = d.mark()
d.undo_to(m)
print(d.components, yes(not d.same(V(0), V(1))))
