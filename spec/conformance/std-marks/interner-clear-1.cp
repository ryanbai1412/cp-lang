// expect: ok, output "YES"
use std.marks.{Interner}
var t = Interner<Str>()
t.id("a")
t.id("b")
t.clear()
print(t.id("b") == 0)
