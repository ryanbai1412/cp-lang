// expect: R0401
// rule: std.fenwick.SparseFenwick2D.add
// A y between the registered ys of an x was never registered, so adding there traps.
use std.fenwick.SparseFenwick2D
var f = SparseFenwick2D<Int>()
f.add_point(1, 2)
f.add_point(1, 4)
f.build()
f.add(1, 3, 1)  // <- here
