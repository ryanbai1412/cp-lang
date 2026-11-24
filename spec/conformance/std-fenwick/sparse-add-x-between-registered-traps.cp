// expect: R0401
// rule: std.fenwick.SparseFenwick2D.add
// An x that falls between registered xs was never registered either, so adding there traps.
use std.fenwick.SparseFenwick2D
var f = SparseFenwick2D<Int>()
f.add_point(1, 2)
f.add_point(3, 2)
f.build()
f.add(2, 2, 1)  // <- here
