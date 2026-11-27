// expect: ok, output "0 12 15 -1\n2 -5 5 0"
// rule: R08.1.9 R08.1.10
// std's `Monoid.of` and `Group.of` take an operator or fn through the impls std declares for
// its `type_of`, not by comparing it with a list of known operators.
use std.algebra.{Monoid, Group, pow}
use std.math.{gcd}
print(Monoid<Int>.of(+).id, Monoid<Int>.of(*).op(3, 4), pow(3, 5, op: +), Monoid<Int>.of(&).id)
print(Monoid<Int>.of(gcd).op(4, 6), Group<Int>.of(+).inv(5), Group<Int>.of(^).inv(5), Monoid<U64>.of(|).id)
