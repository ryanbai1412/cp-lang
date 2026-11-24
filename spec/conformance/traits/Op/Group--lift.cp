// expect: ok, output "5"
use std.algebra.{Group}
let g = Group<Int>(0, (a, b) => a + b, x => -x)
print(g.lift(5))
