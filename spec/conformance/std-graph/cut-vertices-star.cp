// expect: ok, output "YES NO NO NO"
use std.graph.{cut_vertices}
domain V = 0..4
domain E = 0..3
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(0), v: V(2)), (u: V(0), v: V(3))]
let c = cut_vertices(Graph.undirected(es))
print(yes(c[0]), yes(c[1]), yes(c[2]), yes(c[3]))
