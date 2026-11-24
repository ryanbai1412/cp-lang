// expect: ok, output "1 1 0 0\n1 1 0 0"
// rule: std.algebra.Op
// `max` and `min` are `Selective` and `Idempotent` through their own fn-item types; `+` is neither, and `gcd` is not `Idempotent` since `gcd(a, a) == |a|`.
use std.algebra.{Op, Selective, Idempotent}
fn sel<T, M: Op<T>>(m: M, a: T) -> Int:
    if M: Selective<T>: 1 else: 0
fn idem<T, M: Op<T>>(m: M, a: T) -> Int:
    if M: Idempotent<T>: 1 else: 0
print(sel(max, 1), sel(min, 1), sel(+, 1), sel(gcd, 1))
print(idem(max, 1), idem(min, 1), idem(+, 1), idem(gcd, 1))
