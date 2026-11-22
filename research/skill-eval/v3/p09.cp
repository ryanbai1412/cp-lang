use std.graph.{smallest_topo}
let (n, m) = read<(1..=1e5, 1..=2e5)>()
domain V = 1..=n
let g = Graph.directed(read<[m](u: V, v: V)>())
if let Some(order) = smallest_topo(g):
    print(order)
else:
    print("IMPOSSIBLE")
