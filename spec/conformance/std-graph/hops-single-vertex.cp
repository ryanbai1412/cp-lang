// expect: ok, output "0"
// rule: std.graph.Graph.hops
domain V = 0..1
domain E = 0..0
let es: [E](u: V, v: V) = []
let h = Graph.undirected(es).hops()
print(h[V(0), V(0)])
