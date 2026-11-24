// expect: ok, output "6\n9\n2\n-1"
// rule: std.algebra.Op
// `M.Fold` is `T` for an op with an identity and `?T`, `None` on nothing, for one without (decision 456).
use std.algebra.{Op, Monoid}
fn total<T, M: Op<T>>(m: M, xs: List<T>) -> M.Fold:
    let f = m.folds()
    var acc = f.id
    for x in xs: acc = f.op(acc, m.lift(x))
    acc
print(total(Monoid<Int>(0, (a, b) => a + b), [1, 2, 3]))
print(total(+, [4, 5]))
print(total((a: Int, b: Int) => if a > b: a else: b, [1, 2]) ?? -1)
print(total((a: Int, b: Int) => if a > b: a else: b, List<Int>()) ?? -1)
