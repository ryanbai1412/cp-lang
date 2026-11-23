// expect: ok, output "2\n3"
// A range written where a struct's type argument is a domain is an anonymous
// domain over that range (decision 255), like `domain V = 0..3; Dsu<V>()`.
use std.dsu.{Dsu}
var d = Dsu<0..3>()
d.unite(0, 1)
print(d.components)
let n = 5
var e: Dsu<0..n> = Dsu<0..n>()
e.unite(0, 4)
e.unite(1, 2)
print(e.components)
