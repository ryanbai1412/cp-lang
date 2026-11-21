// Unit-weight BFS from vertex 1 over an undirected graph; prints the sum of
// the reachable distances.
let (n, m): (1..=1e6, 0..=2e6) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let dist = bfs(Graph.undirected(edges), from: [V(1)])
var total = 0
for v in V:
    if let d = dist[v]: total += d
print(total)
