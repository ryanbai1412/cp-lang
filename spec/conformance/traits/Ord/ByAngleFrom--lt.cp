// expect: ok, output "YES NO"
use std.geometry.{Pt, ByAngleFrom}
let (l, r) = (ByAngleFrom(Pt(0, 1), Pt(-1, 0)), ByAngleFrom(Pt(0, 1), Pt(1, 0)))
print(yes(l < r), yes(r < l))
