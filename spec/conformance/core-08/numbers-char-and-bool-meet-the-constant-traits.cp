// expect: ok, output "0 1 -9223372036854775807 9223372036854775807 0 9 1 0 1\n6 7 0 1.5000000000"
// rule: 08.2.2
// `Zero`, `One` and `Bounded` are met by the constants they name: numbers have `zero`/`one`, numbers, `Char` and `Bool` have ends, and `Mod<P>` brings its own `zero` and `one`. Integers meet the bit traits.
use std.mod.Mod
type R = 0..10
fn z<T: Zero>(x: T) -> T = T.zero
fn o<T: One>(x: T) -> T = T.one
fn lo<T: Bounded>(x: T) -> T = T.MIN
fn hi<T: Bounded>(x: T) -> T = T.MAX
fn lo_b<T: Bounded>(x: T) -> Int = if x == T.MIN: 1 else: 0
fn both<T: BitAnd + BitOr + BitXor>(a: T, b: T) -> T = (a & b) | (a ^ b)
let r: R = 3
print(z(5), o(5), lo(5), hi(5), lo(r), hi(r), lo_b(false), lo_b('b'), o(Mod<7>(3)).v)
print(both(6, 2), both(3, 4), z(Mod<7>(3)).v, z(2.5) + 1.5)
