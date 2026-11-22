// expect: ok, output "3 4 -1"
// rule: std.graph.Graph.matrix

use std.graph.{Graph}

domain V = 0..2
domain E = 0..3
let es: [E](u: V, v: V, w: Int) = [(u: V(0), v: V(1), w: 5), (u: V(0), v: V(1), w: 3), (u: V(1), v: V(0), w: 4)]
let d = Graph.directed(es).matrix()
print(d[V(0), V(1)] ?? -1, d[V(1), V(0)] ?? -1, d[V(0), V(0)] ?? -1)
