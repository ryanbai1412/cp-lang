// expect: ok, output "YES YES YES"
use std.geometry.{Vec}
let empty: List<Vec> = []
print(yes(polygon_area(empty) == 0.0), yes(polygon_area([Vec(1.0, 1.0)]) == 0.0), yes(polygon_area([Vec(0.0, 0.0), Vec(1.0, 1.0), Vec(2.0, 2.0)]) == 0.0))
