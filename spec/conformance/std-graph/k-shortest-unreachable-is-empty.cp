// expect: ok, output "0"
use std.graph.{k_shortest}
domain V = 0..2
domain E = 0..0
let es: [E](u: V, v: V, w: Int) = []
print(k_shortest(Graph.directed(es), from: [V(0)], to: V(1), k: 3).len)
