// expect: ok, output "YES"
use std.interval.{SparseIntervalSet}
var s = SparseIntervalSet()
s.add(0..3)
print(s.next_gap(1) == 3)
