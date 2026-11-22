// expect: ok, output "YES"
use std.graph.{dominators}
domain V = 0..3
let g: [V]List<V> = [[V(1)], [], [V(1)]]
print(yes(dominators(g, root: V(0)) == [None, Some(V(0)), None]))
