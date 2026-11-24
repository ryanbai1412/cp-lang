// expect: R0401 (checked)
use std.flow.{circulation}
domain V = 0..2
domain E = 0..1
let es: [E](u: V, v: V, lo: Int, hi: Int) = [(u: V(0), v: V(1), lo: 3, hi: 1)]
print(yes(circulation(es) == None)) // <- here
