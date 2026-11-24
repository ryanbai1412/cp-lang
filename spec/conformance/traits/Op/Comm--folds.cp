// expect: ok, output "0 7"
use std.algebra.{Comm, Monoid}
let f = Comm(Monoid<Int>.of(+)).folds()
print(f.id, f.op(2, 5))
