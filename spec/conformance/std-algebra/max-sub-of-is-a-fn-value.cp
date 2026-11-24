// expect: ok, output "7 7"
// A static fn is a fn value like any named fn, so a summary is built with `map(MaxSub.of)` and joined with `+` (decision 472).
use std.algebra.{MaxSub}
let a = [2, -5, 3, 4]
print(a.map(MaxSub.of).fold(+).best, a.map(x => MaxSub.of(x)).fold(+).best)
