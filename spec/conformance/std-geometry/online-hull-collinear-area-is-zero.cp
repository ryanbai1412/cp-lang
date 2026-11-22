// expect: ok, output "0"
use std.geometry.{OnlineHull, Pt}
var h = OnlineHull()
h.add(Pt(0, 0))
h.add(Pt(1, 1))
h.add(Pt(2, 2))
print(h.area2)
