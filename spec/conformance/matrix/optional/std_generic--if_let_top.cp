// expect: ok, output "0 2 1"
use std.graph.{Graph, smallest_topo}
domain V = 0..3
domain E = 0..2
let reqs: [E](u: V, v: V) = [(u: 0, v: 2), (u: 2, v: 1)]
if let y = smallest_topo(Graph.directed(reqs)):
    print(y)
