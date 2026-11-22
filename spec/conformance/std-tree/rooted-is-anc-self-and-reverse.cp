// expect: ok, output "YES YES NO"
use std.tree.{Rooted}
domain V = 0..4
let adj: [V]List<V> = [[V(1)], [V(0), V(2)], [V(1), V(3)], [V(2)]]
let t = Rooted(adj, root: V(0))
print(yes(t.is_anc(V(0), V(3))), yes(t.is_anc(V(2), V(2))), yes(t.is_anc(V(3), V(0))))
