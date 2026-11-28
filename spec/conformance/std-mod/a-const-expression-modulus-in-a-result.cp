// expect: ok, output "6 0"
// rule: 08.2.4
// A `const` worked out from an expression is the modulus in a fn's parameters and result alike.
use std.mod.{Mod}
const P = 3 + 4
fn inc(x: Mod<P>) -> Mod<P> = x + Mod(1)
print(inc(Mod(5)).v, inc(Mod(13)).v)
