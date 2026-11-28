// expect: ok, output "b\na"
// engines: next
use std.marks.{Interner}
var t = Interner<Str>()
t.id("b")
t.id("a")
t.id("b")
for w in t: print(w)
