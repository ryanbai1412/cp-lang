// expect: ok, output "0 1 9\n0 1 2"
use std.dsu.{RollbackArray}
var a = RollbackArray<0..3, Int>(init: i => Int(i))
let m = a.mark()
a.set(2, 9)
print(a)
a.undo_to(m)
print(a)
