// expect: ok, output "p q"
use std.marks.{Arena}
var a = Arena<Str>()
a.alloc("p")
a.alloc("q")
print(a)
