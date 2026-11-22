// expect: ok, output "YES YES\nYES YES\nYES YES\nYES YES\nYES YES\nYES\nYES YES YES\nYES YES"
use std.tree.{Lift}
domain V = 0..4
let par: [V]?(V, Int) = [None, Some((V(0), 5)), Some((V(1), 2)), Some((V(1), 7))]
let l = Lift(par, op: (x, y) => x + y, identity: 0)
print(l.fold(V(2), up_to: V(0)) == 7, l.depth(V(3)) == 2)
print(l.climb(V(2), u => u != V(0)) == (steps: 1, at: V(1)), l.climb(V(2), u => false) == (steps: 0, at: V(2)))
print(l.parent(V(2)) == Some(V(1)), l.parent(V(0)) == None)
print(l.depth(V(0)) == 0, l.depth(V(3)) == 2)
print(l.ancestor(V(3), 2) == Some(V(0)), l.ancestor(V(3), 3) == None)
print(l.lca(V(2), V(3)) == Some(V(1)))
print(l.fold(V(2), up_to: V(1)) == 2, l.fold(V(2), up_to: V(0)) == 7, l.fold(V(2), up_to: V(2)) == 0)
print(l.climb_while(V(2), s => s <= 5) == V(1), l.climb_while(V(2), s => s <= 7) == V(0))
