use std.graph.{smallest_topo}
let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
let es: [0..m](u: V, v: V) = read()
if let order = smallest_topo(Graph.directed(es)): print(order)
else: print(-1)
