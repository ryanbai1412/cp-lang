// expect: L0511 at 5
use std.graph.{Graph}
use std.tree.{Rooted}
domain V = 0..2
let t = Rooted(Graph.undirected([(V(0), V(1))]), V(0))
for v in t.postorder: print(v)
