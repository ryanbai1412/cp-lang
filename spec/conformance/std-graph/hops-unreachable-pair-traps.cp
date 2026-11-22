// expect: R0401 at 4 (checked)
// rule: std.graph.Graph.hops
domain V = 0..3
domain E = 0..1
let es: [E](u: V, v: V) = [(u: V(0), v: V(1))]
let h = Graph.undirected(es).hops()
print(h[V(0), V(1)])
