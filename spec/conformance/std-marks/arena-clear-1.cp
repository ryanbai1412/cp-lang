// expect: ok, output "YES"
use std.marks.{Arena, Id}
var a = Arena<Int>()
a.alloc(5)
a.clear()
print(yes(a.alloc(7) == Id<Int>(0)))
