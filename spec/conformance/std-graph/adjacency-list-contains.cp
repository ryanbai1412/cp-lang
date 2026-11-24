// expect: ok, output "YES NO"
// gap: compiler 2026-11-24T05Z
// rule: std.graph.contains
// `contains(u, v)` works on the adjacency list `Graph.undirected` gives for edges over a plain range.
domain V = 1..=3
let edges: [0..2](u: V, v: V) = [(u: V(1), v: V(2)), (u: V(2), v: V(3))]
let g = Graph.undirected(edges)
print(yes(g.contains(V(3), V(2))), yes(g.contains(V(1), V(3))))
