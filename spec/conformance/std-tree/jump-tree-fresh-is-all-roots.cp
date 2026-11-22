// expect: ok, output "YES YES"
use std.tree.{JumpTree}
domain V = 0..3
let j = JumpTree<V>()
print(yes(j.parent(V(2)) == None), yes(j.root_of(V(2)) == V(2)))
