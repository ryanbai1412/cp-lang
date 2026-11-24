// expect: ok, output "5"
use std.algebra.{Semigroup}
let s = Semigroup<Int>((a, b) => max(a, b))
print(s.lift(5) ?? -1)
