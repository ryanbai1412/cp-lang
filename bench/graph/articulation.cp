// The cut vertices of an undirected simple graph; prints their number and
// the sum of their ids.
use std.graph.{articulation_points}
let (n, m): (1..=1e6, 0..=2e6) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let cut = articulation_points(Graph.undirected(edges))
var (count, total) = (0, 0)
for v in V:
    if cut[v]:
        count += 1
        total += v.to_idx
print(count, total)
