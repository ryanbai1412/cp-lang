// expect: R0401 at 5
use std.graph.{Graph}
domain V = 0..3
domain E = 0..1
let es: [E](u: V, v: V) = [(u: 0, v: 1)]
print(Graph.undirected(es).hops()[0, 1])
