// expect: ok, output "2 1"
// rule: 08.2.5
// `Graph.undirected` infers its arc type from the edges, so `blocks` and `Rooted` work on it.
use std.graph.{Graph, blocks}
use std.tree.{Rooted}
domain V = 1..=3
domain E = 1..=2
let edges: [E](u: V, v: V) = [(u: 1, v: 2), (u: 2, v: 3)]
let g = Graph.undirected(edges)
let (domain B, block) = blocks(g)
let t = Rooted(Graph.undirected(edges), V.first!)
print(B.count, t.children(V.first!).count())
