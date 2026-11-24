// expect: R0401
// rule: std.fenwick.SparseFenwick2D.add
// Adding at an x beyond every registered x is a point the table never made room for, so it traps.
use std.fenwick.SparseFenwick2D
var f = SparseFenwick2D<Int>()
f.add_point(1, 2)
f.build()
f.add(5, 2, 1)  // <- here
