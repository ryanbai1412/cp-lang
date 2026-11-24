// expect: E0310
// tool: check
// has: "its overloads are `sort(by:)` and `sort(cmp:)`"
// Through `use std.seq.*` the call meets the same overloads as through the prelude, so passing
// both labels is the same no-fit error (decisions 87, 624).
use std.seq.*
var a = [2, 1, 3]
a.sort(by: x => -x, cmp: (x, y) => x < y) // <- here
print(a)
