// expect: R0401 at 5
use std.tree.{Rooted}
domain V = 0..2
let adj: [V]List<V> = [[V(1)], [V(0)]]
let from: List<V> = []
print(Rooted.forest(adj, from: from).roots.count())
