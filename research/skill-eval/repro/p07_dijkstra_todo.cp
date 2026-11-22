// Repro: std.graph.dijkstra hits R0402 (`todo` reached) and fails C++ codegen
domain V = 1..=2
domain E = 0..1
let g = Graph.directed([E] _ => (u: V(1), v: V(2), w: 5))
let p = dijkstra(g, from: [(V(1), 0)])
print(p[V(2)]!)
