// expect: ok, output "5 998244352"
// rule: 08.2.4
// A `const`'s name is a static `Int` type argument: `Mod<P1>` is `Mod<998_244_353>`.
use std.mod.{Mod}
const P1 = 998_244_353
fn f(x: Mod<P1>) -> Int = x.v
print(f(Mod<P1>(5)), f(Mod<998_244_353>(-1)))
