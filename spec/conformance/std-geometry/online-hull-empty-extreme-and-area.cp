// expect: ok, output "YES 0"
use std.geometry.{OnlineHull}
let h = OnlineHull()
print(yes(h.extreme(Pt(1, 0)) == None), h.area2)
