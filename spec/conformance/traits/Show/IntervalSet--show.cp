// expect: ok, output "4..6"
use std.interval.{IntervalSet}
var s = IntervalSet<0..8>()
s.add(4..6)
print(s)
