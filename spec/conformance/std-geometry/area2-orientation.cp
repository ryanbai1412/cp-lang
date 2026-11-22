// expect: ok, output "16 -16 0"
use std.geometry.{Pt, area2}
print(area2([Pt(0, 0), Pt(4, 0), Pt(0, 4)]), area2([Pt(0, 0), Pt(0, 4), Pt(4, 0)]), area2([]))
