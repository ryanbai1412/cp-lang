// expect: ok, output "YES"
use std.interval.{SparseIntervalSet}
var s = SparseIntervalSet()
s.add(0..10)
s.remove(3..4)
print(yes(s.runs == 2 and s.covered == 9))
