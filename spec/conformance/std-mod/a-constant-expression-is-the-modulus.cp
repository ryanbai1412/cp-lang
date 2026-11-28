// expect: ok, output "1000000006 1"
// rule: 08.2.4
// A constant expression is a static `Int` type argument, written directly or through a `const`.
use std.mod.{Mod}
const P = 1e9 + 7
fn f(x: Mod<P>) -> Int = x.v
print(f(Mod<1e9 + 7>(-1)), f(Mod<P>(1e9 + 8)))
