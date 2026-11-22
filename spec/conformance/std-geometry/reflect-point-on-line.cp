// expect: ok, output "YES"
use std.geometry.{reflect, Vec}
print(yes(reflect(Vec(1.0, 0.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == Vec(1.0, 0.0)))
