// expect: ok, output "2 0"
use std.marks.{SparseSet}
var s = SparseSet<0..5>()
s.add(2)
s.add(0)
print(s)
