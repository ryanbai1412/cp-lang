// expect: ok, output "YES"
use std.interval.{SparseIntervalSet}
var s = SparseIntervalSet()
s.add(10..12)
s.add(0..3)
print(s.each().collect() == [0..3, 10..12])
