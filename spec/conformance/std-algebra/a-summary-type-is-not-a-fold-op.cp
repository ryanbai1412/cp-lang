// expect: E0302
// A type name is a type, not a value: `fold` takes an operation, so a summary type is folded as `a.map(MaxSub.of).fold(+)` (decision 472).
use std.algebra.{MaxSub}
print([2, -5, 3, 4].fold(MaxSub).best) // <- here
