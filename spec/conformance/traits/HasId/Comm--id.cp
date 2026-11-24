// expect: ok, output "1"
use std.algebra.{Comm, Monoid}
let c = Comm(Monoid<Int>.of(*))
print(c.id)
