// expect: ok, output "0\n2"
// engines: next
use std.geometry.{PointSet, Pt}
var s = PointSet.of([Pt(1, 1), Pt(2, 5), Pt(3, 3)])
s.remove(1)
for i in s: print(i)
