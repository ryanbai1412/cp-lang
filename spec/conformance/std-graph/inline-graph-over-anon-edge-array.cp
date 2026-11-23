// expect: ok, output "0 5 7 | 2 2"
// rule: std.graph.dijkstra
// A graph built inline from an array over an anonymous range domain is one type: the edges'
// domain is the same wherever `Graph.directed` is instantiated, as with `let g` first.
domain V = 0..3
let es: [0..3](u: V, v: V, w: Int) = [(u: V(0), v: V(1), w: 5), (u: V(1), v: V(2), w: 2), (u: V(0), v: V(2), w: 9)]
let d = dijkstra(Graph.directed(es), from: [(V(0), 0)])
let h = dijkstra(Graph.undirected(es), from: [(V(2), 0)])
print(d[V(0)], d[V(1)], d[V(2)], "|", h[V(1)], h[V(2)] + 2)
