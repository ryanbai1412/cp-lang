// expect: ok, output "0..2: 1, 2..3: 9, 3..4: 1"
use std.interval.{IntervalMap}
var m = IntervalMap<0..4, Int>(1)
m.assign(2..3, 9)
print(m)
