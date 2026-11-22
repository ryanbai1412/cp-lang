// expect: R0401 at 2 (checked)
use std.geometry.{reflect, Vec}
print(yes(reflect(Vec(1.0, 1.0), Vec(0.0, 0.0), Vec(0.0, 0.0)) == Vec(1.0, 1.0)))
