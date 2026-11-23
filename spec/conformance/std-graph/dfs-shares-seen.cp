// expect: ok, output "2 0 true"
use std.graph.{dfs}
// `seen:` is read and updated in place, so a second walk over it finds nothing.
domain V = 0..3
let g: [V]List<V> = [[1], [], []]
var seen: [V]Bool = false
let n1 = dfs(g, from: [V(0)], seen: mut seen).count()
let n2 = dfs(g, from: [V(0)], seen: mut seen).count()
print(n1 / 2, n2, seen[V(1)])
