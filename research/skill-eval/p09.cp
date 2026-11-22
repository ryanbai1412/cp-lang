use std.graph.{smallest_topo}

let (n, m) = read<(1..=1e5, 1..=2e5)>()
domain V = 1..=n
domain E = 0..m
let edges = read<[E](u: V, v: V)>()
let g = Graph.directed(edges)
if let order = smallest_topo(g):
    print(order)
else:
    print("IMPOSSIBLE")
