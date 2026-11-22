// expect: ok, output "YES"
use std.tree.{Rooted}
domain V = 0..1
let adj: [V]List<V> = [[]]
let t = Rooted(adj, root: V(0))
let ly = t.layers()
print(yes(ly.level(0).map(b => ly.at[b]).collect() == [V(0)]))
