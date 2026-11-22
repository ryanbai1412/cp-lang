// expect: ok, output "YES"
use std.marks.{Arena}
var a = Arena<Int>()
a.alloc(5)
a.alloc(6)
print(a.len == 2)
