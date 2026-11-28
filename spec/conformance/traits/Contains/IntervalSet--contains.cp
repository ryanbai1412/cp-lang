// expect: ok, output "YES NO"
// engines: next
use std.interval.{IntervalSet}
var s = IntervalSet<0..8>()
s.add(4..6)
print(yes(s.contains(5)), yes(s.contains(6)))
