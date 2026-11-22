// expect: ok, output "YES"
use std.marks.{Interner}
var t = Interner<Str>()
t.id("a")
t.id("b")
t.id("a")
print(yes(t.len == 2))
