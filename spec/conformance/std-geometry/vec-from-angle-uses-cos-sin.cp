// expect: ok, output "1.0000000000 0.0000000000\nYES"
use std.geometry.{Vec, close, PI}
let v = Vec.from_angle(0.0)
print(v.x, v.y)
print(yes(close(Vec.from_angle(PI / 2.0).y, 1.0)))
