// expect: E0203
// rule: 08.1.7
// `Reversible.rev` has no default (decision 599): reversal belongs to the op, so a tuple or record meets `Reversible` only through an impl that says how it reverses.
use std.algebra.{Reversible}
let r = (x: 1).rev() // <- here
print(r.x)
