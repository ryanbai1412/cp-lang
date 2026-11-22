// expect: ok, output "YES"
use std.geometry.{Vec3}
let (a, b) = (Vec3(1.0, 2.0, 3.0), Vec3(3.0, 4.0, 5.0))
let v = a - b
print(yes(v.x == -2.0 and v.y == -2.0 and v.z == -2.0))
