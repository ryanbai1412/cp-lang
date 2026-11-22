// expect: ok, output "3 YES"
use std.tree.{JumpTree}
domain V = 0..4
var j = JumpTree<V>()
j.add(V(1), V(0))
j.add(V(2), V(1))
j.add(V(3), V(2))
print(j.depth(V(3)), yes(j.root_of(V(3)) == V(0)))
