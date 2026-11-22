// expect: ok, output "YES YES YES"
use std.geometry.{polygon_area, Vec}
let none: List<Vec> = []
print(yes(polygon_area(none) == 0.0), yes(polygon_area([Vec(1.0, 1.0)]) == 0.0), yes(polygon_area([Vec(0.0, 0.0), Vec(1.0, 1.0), Vec(2.0, 2.0)]) == 0.0))
