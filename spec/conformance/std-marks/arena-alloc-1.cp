// expect: ok, output "YES"
use std.marks.{Arena, Id}
var a = Arena<Str>()
a.alloc("p")
print(a.alloc("q") == Id<Str>(1))
