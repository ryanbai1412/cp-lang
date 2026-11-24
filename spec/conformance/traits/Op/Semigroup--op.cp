// expect: ok, output "4"
use std.algebra.{Semigroup}
let s = Semigroup<Int>((a, b) => max(a, b))
print(s.op(3, 4))
