// expect: ok, output "NO NO YES"
use std.graph.{bridges}
domain V = 0..3
domain E = 0..3
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(0), v: V(1)), (u: V(1), v: V(2))]
let b = bridges(Graph.undirected(es))
print(yes(b[0]), yes(b[1]), yes(b[2]))
