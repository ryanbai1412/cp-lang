// expect: ok, output "3 2 0"
// rule: std.tree.Lift.of
// `Lift<V, ()>.of(next)` builds a lifting table over unit-valued links, so `[V]?(V, ())` is filled from `next`.
use std.tree.{Lift}
domain V = 0..4
let par: [V]?V = [None, Some(V(0)), Some(V(1)), Some(V(2))]
let l = Lift<V, ()>.of(v => par[v])
print(l.depth(V(3)), l.ancestor(V(3), 1)!.ord, l.lca(V(3), V(0))!.ord)
