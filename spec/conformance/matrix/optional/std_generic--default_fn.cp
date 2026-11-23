// expect: ok, output "0 2 1"
use std.graph.{smallest_topo}
domain V = 0..3
domain E = 0..2
let reqs: [E](u: V, v: V) = [(u: 0, v: 2), (u: 2, v: 1)]
fn f():
    print(smallest_topo(Graph.directed(reqs)) ?? List<V>())
f()
