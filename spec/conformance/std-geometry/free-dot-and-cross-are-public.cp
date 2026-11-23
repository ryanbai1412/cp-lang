// expect: ok, output "YES"
use std.geometry.{cross, dot, Pt}
let ok = dot(Pt(1, 2), Pt(3, 4)) == 11 and dot(Pt(2, -1), Pt(1, 2)) == 0 and dot(Pt(3, 4), Pt(3, 4)) == Pt(3, 4).dot(Pt(3, 4)) and cross(Pt(1, 0), Pt(0, 1)) == 1 and cross(Pt(2, 3), Pt(4, 5)) == Pt(2, 3).cross(Pt(4, 5))
print(yes(ok))
