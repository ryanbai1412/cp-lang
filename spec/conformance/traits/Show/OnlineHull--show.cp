// expect: ok, output "3 YES"
use std.geometry.{OnlineHull, Pt}
var h = OnlineHull()
for p in [Pt(0, 4), Pt(4, 0), Pt(0, 0)]: h.add(p)
print(h.len, yes("{h}" == "{Pt(0, 0)} {Pt(4, 0)} {Pt(0, 4)}"))
