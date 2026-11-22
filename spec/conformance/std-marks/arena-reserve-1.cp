// expect: ok, output "YES"
use std.marks.{Arena}
var a = Arena<Int>()
a.reserve(100)
a.alloc(1)
print(yes(a.len == 1))
