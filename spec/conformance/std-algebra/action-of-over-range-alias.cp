// expect: ok, output "-10\n3 3"
// rule: std.algebra.Action.of
// `Action<M, M>.of` over a range alias resolves the stock action at `M` itself, whether it is given a bare op or the `Monoid<M>` it makes, so its `Action` is the one `of` returns.
use std.algebra.{Action, Monoid, add}
type M = -10..=10
let m = Monoid<M>.of(max)
print(m.id)
print(Action<M, M>.of(add, m).apply(1, 2, 3), Action<M, M>.of(add, max).apply(1, 2, 3))
