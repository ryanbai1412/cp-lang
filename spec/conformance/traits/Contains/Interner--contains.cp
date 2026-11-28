// expect: ok, output "YES NO"
// engines: next
use std.marks.{Interner}
var t = Interner<Str>()
t.id("b")
t.id("a")
t.id("b")
print(yes(t.contains("a")), yes(t.contains("c")))
