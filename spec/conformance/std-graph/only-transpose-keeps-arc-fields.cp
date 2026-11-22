// expect: ok, output "[(to: 0, e: 1, w: 5)] []"
use std.graph.Graph
domain V = 0..3
domain E = 0..3
let es: [E](u: V, v: V, w: Int) = [(u: V(0), v: V(1), w: 4), (u: V(0), v: V(2), w: 5), (u: V(1), v: V(2), w: 6)]
let t = Graph.directed(es).only(v => v != V(1)).transpose()
print([a for a in t.out(V(2))], [a for a in t.out(V(1))])
