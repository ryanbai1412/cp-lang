// expect: ok, output "NO YES"
use std.flow.{ColorMatroid}
domain E = 0..3
domain C = 0..2
let col: [E]C = [C(0), C(0), C(1)]
let lim: [C]Int = [1, 1]
var m = ColorMatroid(col, lim)
m.rebuild([true, false, false])
print(yes(m.can_add(E(1))), yes(m.can_add(E(2))))
