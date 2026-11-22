// expect: ok, output "YES"
use std.marks.{Interner}
var t = Interner<Str>()
t.id("x")
t.id("y")
print(yes(t.key(1) == "y"))
