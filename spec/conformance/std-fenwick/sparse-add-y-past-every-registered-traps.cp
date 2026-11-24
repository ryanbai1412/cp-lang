// expect: R0401
// rule: std.fenwick.SparseFenwick2D.add
// Adding at a y beyond every y registered for that x has nowhere to go, so it traps.
use std.fenwick.SparseFenwick2D
var f = SparseFenwick2D<Int>()
f.add_point(1, 2)
f.build()
f.add(1, 5, 1)  // <- here
