// expect: ok, output "-1 2 5"
use std.algebra.{Semigroup}
let f = Semigroup<Int>((a, b) => max(a, b)).folds()
print(f.id ?? -1, f.op(Some(2), None) ?? -1, f.op(Some(2), Some(5)) ?? -1)
