// expect: ok, output "YES"
use std.geometry.{OnlineHull, Pt}
var h = OnlineHull()
h.add(Pt(3, 4))
print(yes(h.iter().collect() == [Pt(3, 4)]))
