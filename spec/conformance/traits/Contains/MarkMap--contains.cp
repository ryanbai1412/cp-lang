// expect: ok, output "YES NO"
// engines: next
use std.marks.{MarkMap}
var m = MarkMap<0..5, Int>(0)
m.set(3, 7)
print(yes(m.contains(3)), yes(m.contains(1)))
