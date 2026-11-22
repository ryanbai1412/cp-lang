// expect: L0208 at 1
// rule: 11.3.2
use std.graph.{Graph}
domain V = 1..=2
let g = Graph.undirected([(V(1), V(2))])
print(g.deg(V(1)))
