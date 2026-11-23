// expect: ok, output "0:0 0:6 0:5 3:0"
// rule: std.dsu PotentialDsu prints each position's root:offset in V order, without changing the tree
use std.dsu.*
domain V = 0..4
var d = PotentialDsu<V, Int>(op: ^)
d.unite(V(0), V(1), diff: 6)
d.unite(V(1), V(2), diff: 3)
let before = d.diff(V(0), V(2))
print(d)
assert d.diff(V(0), V(2)) == before
