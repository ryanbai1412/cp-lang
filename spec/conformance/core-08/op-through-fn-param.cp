// expect: ok, output "0 1 YES 4 1"
// rule: 08.2.3
use std.algebra.{Monoid}
fn ident<T>(x: T, f: (T, T) -> T) -> T = Monoid<T>.of(f).id
fn both<T>(x: T, f: (T, T) -> T) -> T = f(Monoid<T>.of(f).id, x)
print(ident(0, +), ident(0, *), ident(0, min) == Int.MAX, both(4, *), both(1, max))
