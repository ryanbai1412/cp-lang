// expect: ok, output "0 1"
use std.geometry.{closest_pair2}
print(closest_pair2([Pt(7, 1), Pt(0, 0), Pt(7, 1)]), closest_pair2([Pt(0, 0), Pt(1_000_000, 0), Pt(1_000_001, 0)]))
