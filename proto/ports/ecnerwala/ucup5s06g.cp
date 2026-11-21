// Universal Cup 5 Stage 6 (Hyakumanben) G: vertices on or below a cycle cost
// nothing; every other vertex pays the fewest reversed edges needed to be
// reached from one of them. Print the total, or -1 if some vertex is cut off.
@!multitest
use std.graph.{bfs01}
let (n, m): (1..=5e5, 0..=5e5) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()

// Peel vertices of in-degree zero; whatever keeps an incoming edge is free.
var peeled: [V]Bool = false
for v in peel(Graph.directed(edges)): peeled[v] = true

// Walking an edge forwards is free and backwards costs one.
let dist = bfs01(
    Graph.undirected(edges),
    [v for v in V if not peeled[v]],
    a => if edges[a.e].v == a.to: 0 else: 1,
)
print(if all(dist.reached(v) for v in V): sum(dist[v]! for v in V) else: -1)
