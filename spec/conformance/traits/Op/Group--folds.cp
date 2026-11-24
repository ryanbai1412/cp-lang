// expect: ok, output "0 7"
use std.algebra.{Group}
let f = Group<Int>(0, (a, b) => a + b, x => -x).folds()
print(f.id, f.op(2, 5))
