// expect: ok, output "1 10"
use std.algebra.{Monoid}
let f = Monoid<Int>.of(*).folds()
print(f.id, f.op(2, 5))
