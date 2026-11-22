// expect: ok, output "0"
use std.graph.{Graph}
domain V = 0..2
domain E = 0..0
let es: [E](u: V, v: V) = []
print(Graph.undirected(es).edges().count())
