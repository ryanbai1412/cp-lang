use std.graph.{Graph, bfs}
domain V = 1..=4
let es: List<(V, V)> = [(V(1), V(2)), (V(2), V(3))]
let g = Graph.undirected(es)
let dist = bfs(from: [V(1)], next: u => g[u])
print(dist)
