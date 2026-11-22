// expect: ok, output "YES"
use std.marks.{Interner}
var t = Interner<Str>()
print(t.id("x") == 0 and t.id("y") == 1 and t.id("x") == 0)
