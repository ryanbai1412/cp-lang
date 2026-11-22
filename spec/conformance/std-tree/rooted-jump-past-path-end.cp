// expect: ok, output "0 3 -1"
use std.tree.{Rooted}
domain V = 0..4
let adj: [V]List<V> = [[V(1)], [V(0), V(2)], [V(1), V(3)], [V(2)]]
let t = Rooted(adj, root: V(0))
print(t.jump(V(3), V(0), 3).map(v => Int(v)) ?? -1, t.jump(V(3), V(0), 0).map(v => Int(v)) ?? -1, t.jump(V(3), V(0), 4).map(v => Int(v)) ?? -1)
