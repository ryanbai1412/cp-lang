// The bridges of an undirected graph; prints their number and the sum of
// their edge ids.
use std.graph.{bridges}
let (n, m): (1..=1e6, 0..=2e6) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let bridge = bridges(Graph.undirected(edges))
var (count, total) = (0, 0)
for e in E:
    if bridge[e]:
        count += 1
        total += e.to_idx
print(count, total)
