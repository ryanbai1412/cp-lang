// expect: ok, output "YES NO"
// engines: next
use std.geometry.{PointSet, Pt}
var s = PointSet.of([Pt(1, 1), Pt(2, 5), Pt(3, 3)])
s.remove(1)
print(yes(s.contains(2)), yes(s.contains(1)))
