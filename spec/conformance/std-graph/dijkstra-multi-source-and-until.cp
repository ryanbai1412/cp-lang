// expect: ok, output "0 3 1 5"
// rule: std.graph.dijkstra

domain V = 0..4
domain E = 0..4
let es: [E](u: V, v: V, w: Int) = [
    (u: 0, v: 1, w: 5),
    (u: 0, v: 2, w: 1),
    (u: 2, v: 1, w: 2),
    (u: 1, v: 3, w: 2),
]
let p = dijkstra(Graph.directed(es), from: [(V(0), 0), (V(0), 9)])
print([p[v]! for v in V])
