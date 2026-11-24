// expect: ok, output "0: 1/1, 1: 2/3"
use std.flow.{ColorMatroid}
domain E = 0..4
domain C = 0..2
let col: [E]C = [C(1), C(0), C(1), C(1)]
let lim: [C]Int = [1, 3]
var m = ColorMatroid(col, lim)
m.rebuild([true, true, false, true])
print(m)
