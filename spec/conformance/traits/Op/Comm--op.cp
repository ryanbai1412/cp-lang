// expect: ok, output "7"
use std.algebra.{Comm, Monoid}
let c = Comm(Monoid<Int>.of(+))
print(c.op(3, 4))
