// expect: ok, output "5 2"
// Ranges written as `Fenwick2D`'s type arguments are anonymous domains (decision 255), also
// when the struct uses the parameter only through `.K`.
use std.fenwick.*
var f = Fenwick2D<0..2, 0..3, Int>()
f.add(1, 0, 5)
var g = Fenwick2D<0..2, 0..2, Int>()
g.add(0, 0, 1)
g.add(1, 1, 2)
print(f.fold(1..2, 0..1), g.fold(0..2, 1..2))
