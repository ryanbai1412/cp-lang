// expect: ok, output "2\n0"
// engines: next
use std.marks.{SparseSet}
var s = SparseSet<0..5>()
s.add(2)
s.add(0)
for x in s: print(x)
