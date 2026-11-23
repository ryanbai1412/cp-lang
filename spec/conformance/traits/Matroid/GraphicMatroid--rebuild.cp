// expect: ok, output "YES NO"
use std.flow.{GraphicMatroid}
domain V = 0..3
domain E = 0..3
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(1), v: V(2)), (u: V(0), v: V(2))]
var m = GraphicMatroid(es)
let before = m.can_add(E(2))
m.rebuild([true, true, false])
print(yes(before), yes(m.can_add(E(2))))
