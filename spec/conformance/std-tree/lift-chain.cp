// expect: ok, output "YES YES YES\nYES YES"
use std.tree.{Lift}
domain V = 0..1000
var par: [V]?(V, Int) = None
for v in V:
    if v.ord > 0: par[v] = Some((V((v.ord - 1) / 2), v.ord))
let l = Lift(par, op: max, identity: 0)
print(l.depth(V(999)) == 9, l.lca(V(999), V(998)) == Some(V(61)), l.lca(V(511), V(767)) == Some(V(0)))
print(l.fold(V(999), up_to: V(0)) == 999, l.climb_while(V(999), s => s <= 999) == V(0))
