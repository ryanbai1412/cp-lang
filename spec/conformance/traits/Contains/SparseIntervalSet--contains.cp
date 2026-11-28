// expect: ok, output "YES NO"
// engines: next
use std.interval.{SparseIntervalSet}
var s = SparseIntervalSet()
s.add(4..6)
print(yes(s.contains(5)), yes(s.contains(6)))
