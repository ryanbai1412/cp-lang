// expect: ok, output "-1x + 4, 1x + 0"
use std.dp.{LineContainer, Line}
var c = LineContainer(max)
c.add(Line(1, 0))
c.add(Line(-1, 4))
print(c)
