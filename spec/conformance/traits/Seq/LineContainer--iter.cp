// expect: ok, output "-1 4\n1 0"
// engines: next
use std.dp.{LineContainer, Line}
var c = LineContainer(max)
c.add(Line(1, 0))
c.add(Line(-1, 4))
for l in c: print(l)
