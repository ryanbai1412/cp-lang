// expect: ok, output "5"
use std.algebra.{Monoid}
let m = Monoid<Int>.of(+)
print(m.lift(5))
