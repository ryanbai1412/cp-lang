// expect: ok, output "17 12 0\n2 1\n17"
// rule: 08.2.9
// `Rooted` over a graph whose arcs carry `w` takes edge lengths from them, as every graph
// algorithm does (`arc_len`); plain vertex arcs count 1. A record arc meets `Weighted` by shape.
use std.graph.{Graph, arc_len}
use std.tree.{Rooted}
domain V = 1..=3
domain E = 1..=2
let es: [E](u: V, v: V, w: Int) = [(u: 1, v: 2, w: 5), (u: 2, v: 3, w: 12)]
let t = Rooted(Graph.undirected(es), 1)
print(t.dist(1, 3), t.w(3), t.w(1))
let adj: [V]List<V> = [[2], [1, 3], [2]]
let u = Rooted(adj, 1)
print(u.dist(1, 3), u.w(3))
print(t.diameter().len)
