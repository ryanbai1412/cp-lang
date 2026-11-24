// expect: ok, output "6 6 6 3 c a"
// rule: 07.9.2
// std's `fold` is the one fold: the method `xs.fold(op)` and, imported, the free `fold(xs, op)` run its body over a List, a Range and a Str.
use std.seq.{fold}
print([1, 2, 3].fold(+), (0..4).fold(+), fold([1, 2, 3], +), fold(0..4, max), "bca".fold(Monoid<Char>('a', (x, y) => max(x, y))), fold("bca", Monoid<Char>('z', (x, y) => min(x, y))))
