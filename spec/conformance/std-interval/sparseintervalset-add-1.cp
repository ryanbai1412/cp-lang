// expect: ok, output "YES"
use std.interval.{SparseIntervalSet}
var s = SparseIntervalSet()
s.add(-5..0)
s.add(0..5)
print(s.runs == 1 and s.covered == 10)
