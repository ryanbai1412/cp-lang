// expect: ok, output "YES"
use std.graph.{spanning_forest}
domain V = 0..4
domain E = 0..3
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(2), v: V(3)), (u: V(1), v: V(0))]
print(yes(spanning_forest(Graph.undirected(es)) == [true, true, false]))
