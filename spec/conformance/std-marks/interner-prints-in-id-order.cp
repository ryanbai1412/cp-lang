// expect: ok, output "pear apple fig\n3"
// rule: std.marks.Interner
// An `Interner` prints its keys in id order, so the output never depends on hash order (decision 458).
use std.marks.{Interner}
var names = Interner<Str>()
for w in ["pear", "apple", "pear", "fig", "apple"]: names.id(w)
print(names)
print(names.len)
