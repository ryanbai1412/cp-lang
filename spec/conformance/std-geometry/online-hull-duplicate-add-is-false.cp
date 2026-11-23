// expect: ok, output "YES NO 1"
use std.geometry.{OnlineHull}
var h = OnlineHull()
print(yes(h.add(Pt(0, 0))), yes(h.add(Pt(0, 0))), h.len)
