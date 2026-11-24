// expect: ok, output "YES NO"
use std.geometry.{Pt, ByAngleFrom}
let b = Pt(0, 1)
print(yes(ByAngleFrom(b, Pt(1, 1)) == ByAngleFrom(b, Pt(2, 2))), yes(ByAngleFrom(b, Pt(1, 0)) == ByAngleFrom(b, Pt(-1, 0))))
