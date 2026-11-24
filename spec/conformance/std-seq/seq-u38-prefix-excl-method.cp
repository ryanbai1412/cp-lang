// expect: ok, output "12 8 6 bc ac ab"
// Acceptance for seq/U38 (fold of everything but one position), handbook/product/std/sequences-usecases.md: `a.prefix_excl(op)` is a method, and a non-commutative op keeps the order before `i` then after it.
use std.prefix.{prefix_excl}
print([2, 3, 4].prefix_excl(*), ["a", "b", "c"].prefix_excl(Monoid("", (x: Str, y: Str) => x + y)))
