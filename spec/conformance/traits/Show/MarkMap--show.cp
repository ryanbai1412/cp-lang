// expect: ok, output "1=9, 3=7"
use std.marks.{MarkMap}
var m = MarkMap<0..5, Int>(0)
m.set(3, 7)
m.set(1, 9)
print(m)
