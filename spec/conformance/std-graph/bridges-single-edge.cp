// expect: ok, output "YES"
use std.graph.{Graph, bridges}
domain V = 0..2
domain E = 0..1
let es: [E](u: V, v: V) = [(u: V(0), v: V(1))]
let b = bridges(Graph.undirected(es))
print(yes(b[0]))
