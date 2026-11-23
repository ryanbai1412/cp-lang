// expect: ok, output "YES NO"
use std.flow.{GraphicMatroid}
domain V = 0..3
domain E = 0..3
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(1), v: V(2)), (u: V(0), v: V(2))]
var m = GraphicMatroid(es)
m.rebuild([true, false, false])
print(yes(m.can_add(E(1))), yes(m.can_add(E(0))))
