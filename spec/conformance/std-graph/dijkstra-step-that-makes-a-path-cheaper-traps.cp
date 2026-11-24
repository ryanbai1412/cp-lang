// expect: R0401
// rule: std.graph.dijkstra

domain V = 0..2
domain E = 0..1
let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 3)]
let p = dijkstra(Graph.directed(es), from: [(V(0), 5)], step: min)  // <- here
print(p[1]!)
