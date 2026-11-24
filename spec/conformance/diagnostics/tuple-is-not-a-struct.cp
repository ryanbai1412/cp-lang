// expect: E0302
// rule: 08.1.6
// A tuple is not a `Pt`, even one of two `Int`s: a named struct is nominal.
use std.geometry.{Pt}
fn norm(a: Pt) -> Int = a.norm2
let p = (3, 4)
print(norm(p)) // <- here
