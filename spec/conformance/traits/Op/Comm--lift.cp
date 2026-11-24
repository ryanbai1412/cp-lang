// expect: ok, output "5"
use std.algebra.{Comm, Monoid}
let c = Comm(Monoid<Int>.of(+))
print(c.lift(5))
