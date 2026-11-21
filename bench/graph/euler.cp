// An Euler circuit of an undirected graph (Hierholzer); prints where it
// starts and the sum of (i + 1) * edge(i).
use std.graph.{euler_walk}
let (n, m): (1..=1e6, 1..=1e6) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let walk = euler_walk(edges, directed: false)!
var total = 0
for i in 0..walk.arcs.len: total += (i + 1) * walk.arcs[i].e.to_idx
print(walk.start.to_idx, total)
