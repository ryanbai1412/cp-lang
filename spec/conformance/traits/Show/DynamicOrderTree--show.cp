// expect: ok, output "6 4 1"
use std.segtree.{DynamicOrderTree}
var t = DynamicOrderTree([4, 4, 1])
t.set(0, 6)
print(t)
