// expect: ok, output "3\n6\n10\n5"
// rule: 06.2.1
// A closure capturing a generic param bound to an operator sees that operator, however deep it nests.
use std.algebra.{Monoid}
fn mk<A>(a: A) -> (Int, Int) -> Int:
    (x: Int, y: Int) => Monoid<Int>.of(a).op(x, y)
fn twice<A>(a: A) -> (Int) -> Int: (x: Int) => Monoid<Int>.of(a).op(x, x)
fn deep<A>(a: A) -> (Int) -> Int: (x: Int) => ((y: Int) => Monoid<Int>.of(a).op(x, y))(10)
fn shadow<A>(a: A) -> (Int) -> Int: (a: Int) => a + 1
print(mk(min)(3, 4))
print(twice(+)(3))
print(deep(max)(4))
print(shadow(min)(4))
