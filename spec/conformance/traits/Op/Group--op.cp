// expect: ok, output "7 0"
use std.algebra.{Group}
let g = Group<Int>(0, (a, b) => a + b, x => -x)
print(g.op(3, 4), g.op(3, g.inv(3)))
