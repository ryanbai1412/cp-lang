// expect: ok, output "NO"
use std.graph.{Graph, negative_cycle}
domain V = 0..3
domain E = 0..2
let es: [E](u: V, v: V, w: Int) = [(u: V(0), v: V(1), w: -5), (u: V(1), v: V(2), w: -5)]
print(yes(negative_cycle(Graph.directed(es)) != None))
