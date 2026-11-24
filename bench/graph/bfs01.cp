// 0-1 BFS from vertex 1 over an undirected graph with 0/1 weights; prints the
// sum of the reachable distances.
use std.graph.{bfs01}
let (n, m): (1..=1e6, 0..=2e6) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V, w: 0..=1) = read()
let dist = bfs01(Graph.undirected(edges), from: [V(1)])
var total = 0
for v in V:
    if let d = dist[v]: total += d
print(total)
