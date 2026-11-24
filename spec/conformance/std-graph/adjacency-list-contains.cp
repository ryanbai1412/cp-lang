// expect: ok, output "YES NO"
// rule: std.graph.contains
// An arc test on the adjacency list `Graph.undirected` gives for edges over a plain range asks `u`'s list.
domain V = 1..=3
let edges: [0..2](u: V, v: V) = [(u: V(1), v: V(2)), (u: V(2), v: V(3))]
let g = Graph.undirected(edges)
print(yes(g[V(3)].contains(V(2))), yes(g[V(1)].contains(V(3))))
