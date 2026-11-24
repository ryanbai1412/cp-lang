// expect: ok, output "100000001000000 200000000000000"
// Contest-sized lines (slopes near 1e9, intercepts near 1e14) are compared in I128, so dropping a covered line can't overflow.
use std.dp.{MonotoneHull, Line}
var h = MonotoneHull<()>(max)
h.add(Line(1_000_000, 100_000_000_000_000))
h.add(Line(2_000_000_000, 0))
h.add(Line(3_000_000_000, -100_000_000_000_000))
print(h.at(1)!, h.at(100_000)!)
