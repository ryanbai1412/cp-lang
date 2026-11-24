// expect: ok, output "left of 1.0000000000 2.0000000000 along 0.0000000000 1.0000000000"
use std.geometry.{Vec, HalfPlane}
print(HalfPlane.through(Vec(1.0, 2.0), Vec(1.0, 3.0)))
