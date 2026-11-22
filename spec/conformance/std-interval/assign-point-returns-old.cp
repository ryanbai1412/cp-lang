// expect: ok, output "1 5 2"
use std.interval.{IntervalMap}
domain D = 0..2
let a: [D]Int = [D] v => [1, 2][Int(v)]
var m = IntervalMap<D, Int>.from(a)
let old = m.assign_point(D(0), 5)
print(old, m.get(D(0)), m.get(D(1)))
