// expect: ok, output "17 2\n5 1 5\n36"
// rule: 08.2.9
// `g.weighted(f)` reads each arc's `w` as `f(a)` and passes `.to` and `.e` through, so a
// graph whose edges keep their length under another name still gives `Rooted` and
// `dijkstra` their lengths. The view stores nothing.
use std.graph.{Graph, dijkstra}
use std.tree.{Rooted}
domain V = 1..=3
domain E = 1..=2
let es: [E](u: V, v: V, len: Int) = [(u: 1, v: 2, len: 5), (u: 2, v: 3, len: 12)]
let g = Graph.undirected(es).weighted(a => es[a.e].len)
let t = Rooted(g, 1)
print(t.dist(1, 3), t.depth(3))
let a = g.out(1).collect()[0]
print(a.w, a.e, es[a.e].len)
print(dijkstra(Graph.undirected(es).weighted(a => 2 * es[a.e].len + 1), from: [(V(1), 0)])[3]!)
