// expect: ok, output "YES"
use std.geometry.{Pt, minkowski_upper}
print(yes(minkowski_upper([Pt(1, 2)], [Pt(3, 4)]) == [Pt(4, 6)]))
