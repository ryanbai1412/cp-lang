// expect: ok, output "YES"
use std.geometry.{Vec}
print(yes(Vec(1.0, 2.0).rot(Vec(1.0, 0.0)) == Vec(1.0, 2.0)))
