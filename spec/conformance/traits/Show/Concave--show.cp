// expect: ok, output "concave on 0..=2: -2 -1 0"
use std.slope.{Concave}
var f = Concave.flat(0..3)
f.add_left(2)
print(f)
