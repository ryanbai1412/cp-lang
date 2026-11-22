// expect: ok, output "1 1"
use std.interval.{IntervalMap}
domain D = 0..1
let m = IntervalMap<D, Int>(7)
print(m.runs, IntervalMap<D, Int>(0).runs)
