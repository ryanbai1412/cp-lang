// expect: ok, output "YES"
use std.geometry.{Vec}
let (a, b) = (Vec(1.0, 2.0), Vec(3.0, 4.0))
let v = a - b
print(yes(v.x == -2.0 and v.y == -2.0))
