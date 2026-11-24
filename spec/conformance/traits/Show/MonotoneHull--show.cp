// expect: ok, output "2x + 0, -1x + 3"
use std.dp.{MonotoneHull, Line}
var h = MonotoneHull<()>(min)
h.add(Line(2, 0))
h.add(Line(-1, 3))
print(h)
