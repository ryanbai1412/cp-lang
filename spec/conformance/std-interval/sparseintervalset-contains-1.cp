// expect: ok, output "YES"
use std.interval.{SparseIntervalSet}
var s = SparseIntervalSet()
s.add(10..20)
print(s.contains(19) and not s.contains(20))
