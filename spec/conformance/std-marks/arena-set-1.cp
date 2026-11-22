// expect: ok, output "YES"
use std.marks.{Arena}
var a = Arena<Int>()
let i = a.alloc(5)
a.set(i, 9)
print(yes(a[i] == 9))
