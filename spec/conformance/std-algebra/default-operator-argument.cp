// expect: ok, output "3 7"
// rule: 06.1.5 08.1.9
// An omitted parameter whose default is a bare operator (`over: M = (+)`) binds `M` to that
// operator exactly as passing `over: +` would, so `Monoid.of` sees `+`, not a fn value; `M` is
// bound by `FromOp<Int>`, the trait `Monoid.of` takes its op through.
use std.algebra.{FromOp}
fn total<M: FromOp<Int>>(xs: List<Int>, over: M = (+)) -> Int = xs.fold(Monoid<Int>.of(over))
let a = Action<Int, Int>(apply: (d, x, n) => x + d * n)
print(total([1, 2]), a.over.op(3, 4))
