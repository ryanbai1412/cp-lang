// expect: ok, output "1"
use std.dsu.{RollbackArray}
domain D = 0..2
var a = RollbackArray<D, Int>(init: i => Int(i) + 1)
let m = a.mark()
a.undo_to(m)
print(a.get(D(0)))
