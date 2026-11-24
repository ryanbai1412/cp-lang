// expect: ok, output "abcde 6 5\n6 6 6 10"
// rule: 07.7.1
// `xs.fold(op)` is std's seq method: a user monoid, a bare operator, `max` and `gcd` all fold
// a list, a slice, a `[D]T` and a range.
use std.algebra.{Monoid}
use std.math.{gcd}
let cat = Monoid<Str>("", (a, b) => a + b)
print(["ab", "c", "de"].fold(cat), [1, 2, 3].fold(+), [1, 5, 3].fold(max))
let xs = [12, 18, 30]
let a: [0..3]Int = [12, 18, 30]
print(xs.fold(gcd), xs[1..].fold(gcd), a.fold(gcd), (1..=4).fold(+))
