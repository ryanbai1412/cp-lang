domain V = 0..3
var jt = JumpTree<V>.new()
jt.add(V(1), V(0))
jt.add(V(2), V(1))
print(jt.ancestor(V(2), 2))
