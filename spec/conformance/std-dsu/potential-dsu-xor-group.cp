// expect: ok, output "5 -1 0 1 5"
// rule: std.dsu PotentialDsu(op: ^) keeps xor potentials: `row ^ col == v` constraints, contradictions and rollback
use std.dsu.*
domain V = 0..4
var d = PotentialDsu<V, Int>(op: ^)
d.unite(V(0), V(1), 6)
d.unite(V(1), V(2), 3)
let m = d.mark()
let bad = d.unite(V(2), V(0), 4)
let c = d.consistent
d.undo_to(m)
print(d.diff(V(0), V(2)) ?? -1, d.diff(V(0), V(3)) ?? -1, (if bad or c: 1 else: 0), (if d.consistent: 1 else: 0), d.diff(V(2), V(0)) ?? -1)
