// expect: ok, output "b a"
use std.marks.{Interner}
var t = Interner<Str>()
for w in ["b", "a", "b"]: t.id(w)
print(t)
