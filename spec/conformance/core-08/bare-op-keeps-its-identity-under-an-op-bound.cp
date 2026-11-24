// expect: ok, output "YES NO"
// rule: 08.1.9
// Under an `M: Op<E>` bound a bare operator keeps its own fn-item type, so `same(op, +)` still tells `+` from `max` (R08.1.9).
use std.algebra.Op
fn same<A, B>(a: A, b: B) -> Bool = a == b
fn is_add<M: Op<E>, E>(op: M, x: E) -> Bool = same(op, +)
print(yes(is_add(+, 1)), yes(is_add(max, 1)))
