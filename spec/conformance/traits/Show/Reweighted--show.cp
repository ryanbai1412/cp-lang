// expect: ok, output "0->2:3 1->2:4"
use std.graph.{Graph}
domain V = 0..3
domain E = 0..2
let es: [E](u: V, v: V) = [(u: 0, v: 2), (u: 1, v: 2)]
print(Graph.directed(es).weighted(a => 3 + a.e.ord))
