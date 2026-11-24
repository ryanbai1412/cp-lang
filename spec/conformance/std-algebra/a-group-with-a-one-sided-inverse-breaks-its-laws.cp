// expect: R0401
// rule: std.algebra.Group
// On the sample 1, 0 is a two-sided identity and `op(1, 2)` is 0, but `op(2, 1)` is 1, so `law inverse` fails.
use std.algebra.Group
fn op(a: Int, b: Int) -> Int:
    if a == 0: return b
    if b == 0: return a
    if a == 1 and b == 2: return 0
    1
assert laws_hold(Group<Int>(0, op, a => 3 - a), [1])  // <- here
