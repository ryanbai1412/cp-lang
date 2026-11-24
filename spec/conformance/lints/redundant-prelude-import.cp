// expect: L0208
// rule: 11.3.2
use std.graph.{Graph} // <- here
domain V = 1..=2
let g = Graph.undirected([(V(1), V(2))])
print(g.deg(V(1)))
