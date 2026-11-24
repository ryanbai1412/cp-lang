// expect: R0401
use std.segtree.{DynamicOrderTree}
var t = DynamicOrderTree([3, 1, 4], values: [1, 3, 4])
t.set(0, 2)  // <- here
print(t)
