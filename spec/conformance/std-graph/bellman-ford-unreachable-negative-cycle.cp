// expect: ok, output "1 YES"
use std.graph.{Graph, bellman_ford}
domain V = 0..4
domain E = 0..3
let es: [E](u: V, v: V, w: Int) = [(u: V(0), v: V(1), w: 1), (u: V(2), v: V(3), w: -5), (u: V(3), v: V(2), w: 1)]
let p = bellman_ford(Graph.directed(es), from: [(V(0), 0)])
print(p![1]!, yes(p![2] == None))
