// expect: ok, output "0 2 1"
// rule: 01.6.2
use std.graph.{smallest_topo}
domain V = 0..3
domain E = 0..2
let reqs: [E](u: V, v: V) = [(u: 0, v: 2), (u: 2, v: 1)]
let order = smallest_topo(Graph.directed(reqs)) else:
    print("cycle")
    return
print(order[0], order[1], order[2])
