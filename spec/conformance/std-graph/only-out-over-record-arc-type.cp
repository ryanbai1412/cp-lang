// expect: ok, output "3\n2\n1 1"
// rule: 08.2.5
// A graph whose arcs carry their road keeps them through `only`'s generic `out`, `blocks` and `Rooted`, with the arc type inferred from the edges.
use std.graph.{Graph, blocks}
use std.tree.{Rooted}
domain Spot = 1..=3
domain Road = 1..=2
let roads: [Road](u: Spot, v: Spot) = [(u: 1, v: 2), (u: 2, v: 3)]
let g = Graph.undirected(roads)
for a in g.only(keep: s => s != Spot(1)).out(Spot(2)): print(a.to.ord + 1)
for a in g.only().out(Spot(3)): print(a.to.ord + 1)
let (domain B, block) = blocks(g)
let t = Rooted(Graph.undirected(roads), Spot.first!)
print(B.count - 1, t.children(Spot.first!).count())
