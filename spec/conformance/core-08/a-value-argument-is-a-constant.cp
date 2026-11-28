// expect: E0302
// rule: 08.2.4
// engines: next
// has: "type Z = Mod(p)"
// A value argument is a constant; a modulus read from input is `type Z = Mod(p)`'s (decision 232).
use std.mod.{Mod}
let p: Int = read()
let x: Mod<p> = Mod(7) // <- here
print(x)
