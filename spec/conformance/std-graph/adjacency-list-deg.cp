// expect: ok, output "1 2 1"
// rule: std.graph deg
// input: "3\n1 2\n2 3\n"
// `Graph.undirected` over a plain range of edges gives a `[V]List<V>`, which has `deg` like a `Graph`.
let n: 2..=5 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)
print(g.deg(V(1)), g.deg(V(2)), g.deg(V(3)))
