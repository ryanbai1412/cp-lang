// expect: ok, output "YES"
use std.geometry.{min_enclosing_circle, Circle, Vec}
print(yes(min_enclosing_circle([Vec(2.0, 3.0)]) == Circle(Vec(2.0, 3.0), 0.0)))
