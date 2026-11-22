// expect: ok, output "0 2 1"
use std.graph.{Graph, smallest_topo}
domain V = 0..3
domain E = 0..2
let reqs: [E](u: V, v: V) = [(u: 0, v: 2), (u: 2, v: 1)]
var left = 1
while let y = (if left > 0: smallest_topo(Graph.directed(reqs)) else: None):
    left -= 1
    print(y)
