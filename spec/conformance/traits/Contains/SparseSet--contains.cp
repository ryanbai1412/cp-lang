// expect: ok, output "YES NO"
// engines: next
use std.marks.{SparseSet}
var s = SparseSet<0..5>()
s.add(2)
s.add(0)
print(yes(s.contains(2)), yes(s.contains(1)))
