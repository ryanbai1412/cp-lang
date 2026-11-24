// expect: ok, output "by angle 2 5 from 0 1"
use std.geometry.{Pt, ByAngleFrom}
print(ByAngleFrom(Pt(0, 1), Pt(2, 5)))
