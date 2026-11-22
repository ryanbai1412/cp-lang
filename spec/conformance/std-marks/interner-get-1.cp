// expect: ok, output "YES"
use std.marks.{Interner}
var t = Interner<Str>()
t.id("x")
print(yes(t.get("x") == Some(0) and t.get("y") == None and t.len == 1))
