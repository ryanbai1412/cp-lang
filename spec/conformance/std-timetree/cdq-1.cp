// expect: ok, output "YES"
use std.timetree.{cdq}
var pairs = 0
cdq([1, 2, 3], on_cross: (l, r) => pairs += l.len * r.len)
print(pairs == 3)
