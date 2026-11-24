// expect: E0310
// tool: check
// has: "no `seq.sort` takes `by:` and `cmp:`: its overloads are `seq.sort(by:)` and `seq.sort(cmp:)`"
// Through the module, `seq.sort`, the call fits no overload as the method and free forms don't (decision 624).
use std.seq
var a = [2, 1, 3]
seq.sort(mut a, by: x => -x, cmp: (x, y) => x < y) // <- here
print(a)
