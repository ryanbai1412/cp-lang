// expect: ok, output "8 12"
// rule: 08.2.11
// `twice<T, M: Op<T>>`: a lambda given for `M` takes `(T, T) -> T` from `Op`'s impl for fn types, with the `T` the later `a` fixes (G7.3).
use std.algebra.{Op}
fn twice<T, M: Op<T>>(m: M, a: T) -> T = m.op(a, a)
print(twice((x, y) => x + y, 4), twice((x, y) => x * y + 3, 3))
