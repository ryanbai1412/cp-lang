// expect: R0401
// rule: std.algebra.Monoid.reversed
// A monoid made without `rev:` has no way to read a fold backwards, so asking traps instead of guessing.
use std.algebra.Monoid
let m = Monoid<Int>(0, (a, b) => a * 10 + b)  // <- here
print(m.reversed(12))
