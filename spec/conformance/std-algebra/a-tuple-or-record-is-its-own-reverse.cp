// expect: ok, output "3 4 1"
// rule: 08.1.7
// `Reversible` leaves nothing without a body, so every tuple and record meets it by shape and takes its default `rev`, the identity.
use std.algebra.{Reversible}
let r = (x: 1).rev()
print((3, 4).rev(), r.x)
