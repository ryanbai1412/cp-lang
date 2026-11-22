// expect: ok, output "YES"
use std.geometry.{Vec}
let (a, b) = (Vec(1.0, 2.0), Vec(3.0, 4.0))
let v = a / 2.0
print(yes(v.x == 0.5 and v.y == 1.0))
