// expect: ok, output "mark 2"
use std.dsu.{RollbackArray}
var a = RollbackArray<0..2, Int>(init: _ => 0)
a.set(0, 5)
a.set(1, 6)
print(a.mark())
