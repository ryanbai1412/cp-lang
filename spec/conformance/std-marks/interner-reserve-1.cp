// expect: ok, output "YES"
use std.marks.{Interner}
var t = Interner<Int>()
t.reserve(100)
t.id(5)
print(t.len == 1)
