// expect: ok, output "YES"
use std.geometry.{Pt, minkowski_sum}
print(yes(minkowski_sum([Pt(1, 2)], [Pt(3, 4)]) == [Pt(4, 6)]))
