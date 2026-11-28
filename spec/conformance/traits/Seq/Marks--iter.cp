// expect: ok, output "1\n3"
// engines: next
use std.marks.{Marks}
var m = Marks<0..5>()
m.add(3)
m.add(1)
for x in m: print(x)
