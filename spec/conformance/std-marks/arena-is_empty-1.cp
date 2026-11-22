// expect: ok, output "YES"
use std.marks.{Arena}
var a = Arena<Int>()
a.alloc(5)
a.clear()
print(yes(a.is_empty))
