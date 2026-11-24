// expect: ok, output "3\n4"
// rule: 04.2.11
// A cell of `[P] p => ..` is a position of `P`, so it keeps `.ord`, also where `P` is a std body's generic `V`.
use std.flow.max_closure
domain R = 0..2
domain C = 0..2
domain P = (R, C)
let ok: [P]Bool = true
let t: [P]Int = [P] p => p.ord
print(t[P(R(1), C(1))])
print(max_closure(before: (p: P) => [p].filter(q => ok[q]), profit: p => 1).value)
