// expect: ok, output "YES NO"
// engines: next
use std.marks.{Marks}
var m = Marks<0..5>()
m.add(3)
m.add(1)
print(yes(m.contains(3)), yes(m.contains(2)))
