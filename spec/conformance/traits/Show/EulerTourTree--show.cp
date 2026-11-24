// expect: ok, output "1 2 3 8"
use std.tree.{EulerTourTree}
var t = EulerTourTree<0..4, Int>(+, a: [1, 2, 3, 4])
t.link(0, 1)
t.link(1, 2)
t.set(3, 8)
print(t)
