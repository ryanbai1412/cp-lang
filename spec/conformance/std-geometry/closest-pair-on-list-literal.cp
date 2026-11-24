// expect: ok, output "0 2 1\nYES"
use std.geometry.{Pt}
let r = closest_pair([Pt(0, 0), Pt(5, 5), Pt(1, 0)])
print(r.a, r.b, r.dist2)
print(yes(r == (0, 2, 1)))
