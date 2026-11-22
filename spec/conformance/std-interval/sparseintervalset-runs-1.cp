// expect: ok, output "YES"
use std.interval.{SparseIntervalSet}
var s = SparseIntervalSet()
s.add(0..3)
s.add(10..12)
print(s.runs == 2)
