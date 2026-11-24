// expect: ok, output "0..3=0, 3..5=1, 5..10=0"
use std.interval.{SparseIntervalMap}
var m = SparseIntervalMap(0, 10, 0)
m.assign(3..5, 1)
print("{m}")
