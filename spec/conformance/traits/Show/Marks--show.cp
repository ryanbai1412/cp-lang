// expect: ok, output "1 3"
use std.marks.{Marks}
var m = Marks<0..5>()
m.add(3)
m.add(1)
print(m)
