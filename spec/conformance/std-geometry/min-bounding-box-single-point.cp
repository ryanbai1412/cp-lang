// expect: ok, output "YES"
use std.geometry.{min_bounding_box}
let b = min_bounding_box([Pt(1, 1)])
print(yes(b.area == 0.0 and b.perimeter == 0.0))
