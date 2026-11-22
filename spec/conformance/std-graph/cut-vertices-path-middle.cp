// expect: ok, output "NO YES NO"
use std.graph.{Graph, cut_vertices}
domain V = 0..3
domain E = 0..2
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(1), v: V(2))]
let c = cut_vertices(Graph.undirected(es))
print(yes(c[0]), yes(c[1]), yes(c[2]))
