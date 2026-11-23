// expect: ok, output "7 YES"
// rule: std.graph.dijkstra
domain V = 0..4
domain E = 0..2
let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 2)]
let p = dijkstra(Graph.directed(es), from: [(V(0), 0)])
print(p[V(2)], yes(p.get(V(3)) == None))
