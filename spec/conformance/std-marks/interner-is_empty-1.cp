// expect: ok, output "YES"
use std.marks.{Interner}
var t = Interner<Str>()
t.id("a")
t.clear()
print(t.is_empty)
