// expect: ok, output "5 -1"
// rule: std.graph.dijkstra

use std.graph.{Graph, dijkstra}

domain V = 1..=3
domain E = 0..3
let es: [E](u: V, v: V, w: Int) = [(u: V(1), v: V(2), w: 4), (u: V(2), v: V(3), w: 1), (u: V(1), v: V(3), w: 10)]
let g = Graph.undirected(es)
let p = dijkstra(g, from: [(V.first!, 0)])

domain V2 = 1..=2
domain E2 = 0..0
let es2: [E2](u: V2, v: V2, w: Int) = []
let g2 = Graph.undirected(es2)
let p2 = dijkstra(g2, from: [(V2.first!, 0)])

print(p.dist[V.last!] ?? -1, p2.dist[V2.last!] ?? -1)
