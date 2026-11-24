// expect: ok, output "3\n6\n-1\n9\n3"
// rule: 08.1.9
// A struct's bound reaches its methods: `M.Fold` is the fold its op's impl chooses. A bare `max` or `+` at the constructor takes its `Monoid` impl once `xs` gives `T`, so it folds to `T`; a closure stays a fn and folds to `?T` (decision 456).
use std.algebra.{Op, Monoid}
struct W<T, M: Op<T>>(op: M, xs: List<T>)
impl<T, M: Op<T>> W<T, M>:
    fn all(self) -> M.Fold:
        var acc = self.op.folds().id
        for x in self.xs: acc = self.op.folds().op(acc, self.op.lift(x))
        acc
let top: Int = W(max, [3, 1, 2]).all()
print(top)
let c = W((a: Int, b: Int) => a + b, [3, 1, 2])
print(c.all() ?? -1)
let e = W((a: Int, b: Int) => a + b, [])
print(e.all() ?? -1)
print(W(Monoid<Int>(0, (a, b) => a + b), [4, 5]).all())
let w: Int = W(+, [1, 2]).all()
print(w)
