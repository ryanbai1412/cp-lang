// expect: ok, output "assoc ident YES NO"
// rule: 08.3.5
// A program's impl of a std trait that declares laws gets the trait's `laws`: each law runs on
// the impl's own fns, so `laws_hold` finds the subtraction that is no monoid.
use std.algebra.Monoid
let add = Monoid<Int>.of(+)
let sub = Monoid<Int>(0, (a, b) => a - b)
print(add.laws[0].name, add.laws[1].name, yes(add.laws[0].holds([1, 2, 3], [0, 1, 2])), yes(sub.laws[0].holds([1, 2, 3], [0, 1, 2])))
