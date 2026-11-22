// expect: ok, output "5 0"
use std.dsu.{RollbackArray}
domain D = 0..2
var a = RollbackArray<D, Int>(init: _ => 0)
let m0 = a.mark()
a.set(D(0), 5)
let m1 = a.mark()
a.set(D(0), 9)
a.undo_to(m1)
let mid = a.get(D(0))
a.undo_to(m0)
print(mid, a.get(D(0)))
