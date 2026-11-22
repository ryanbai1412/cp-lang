// expect: ok, output "YES"
use std.marks.{Interner}
var t = Interner<Int>()
t.id(40)
print(t.contains(40) and not t.contains(41))
