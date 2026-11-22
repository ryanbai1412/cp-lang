// expect: ok, output "YES"
use std.interval.{SparseIntervalSet}
var s = SparseIntervalSet()
s.add(0..3)
print(yes(s.next_gap(1) == 3))
