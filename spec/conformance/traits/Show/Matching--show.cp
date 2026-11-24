// expect: ok, output "0-1, 1-0"
use std.flow.{Matching}
var m = Matching<0..2, 0..2>()
m.add(0, 1)
m.add(1, 0)
m.augment_all()
print(m)
