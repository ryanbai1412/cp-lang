// expect: E0302
// rule: 08.2.4
// engines: next
// `Mod` takes one value, and `5..=5` is a range type, not the value 5: `Mod<5..=5>` is not `Mod<5>`.
use std.mod.{Mod}
print(Mod<5>(7))
print(Mod<5..=5>(7)) // <- here
