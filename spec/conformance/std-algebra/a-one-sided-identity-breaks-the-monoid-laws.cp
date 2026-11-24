// expect: R0401
// rule: std.algebra.Monoid
// `(a, b) => b` is associative and 0 is a left identity, but `op(a, 0)` is 0, not `a`, so `law ident` fails.
use std.algebra.Monoid
assert laws_hold(Monoid<Int>(0, (a, b) => b), [1, 2])  // <- here
