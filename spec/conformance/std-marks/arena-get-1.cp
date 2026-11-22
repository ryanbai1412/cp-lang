// expect: ok, output "YES"
use std.marks.{Arena}
var a = Arena<Int>()
let i = a.alloc(5)
a.alloc(6)
print(yes(a.get(i) == 5))
