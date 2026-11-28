// expect: E0302
// rule: 08.2.4
// engines: next
// has: "type Z = Mod(p)"
// A signature's value argument is a constant too, never a `let` read from input.
use std.mod.{Mod}
let p: Int = read()
fn f(x: Mod<p>) -> Int = x.v // <- here
print(f(7))
