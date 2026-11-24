// expect: R0401
// rule: std.algebra.Group
// On the sample 1, `op` is associative and 2 is a two-sided inverse, but `op(1, 0)` is 5, so `law ident` fails.
use std.algebra.Group
fn op(a: Int, b: Int) -> Int:
    if a == 1 and b == 0: return 5
    if a == 0: return b
    if a + b == 3: return 0
    1
assert laws_hold(Group<Int>(0, op, a => 3 - a), [1])  // <- here
