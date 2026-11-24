// expect: ok, output "7"
use std.algebra.{Monoid}
let m = Monoid<Int>.of(+)
print(m.op(3, 4))
