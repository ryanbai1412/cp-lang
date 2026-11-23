// expect: ok, output "0 1 0\n1 0 1"
// rule: std.graph transpose, std.tree spanning_tree
// Edges indexed by a plain range give an adjacency list; it has `transpose` and `spanning_tree` as a `Graph` does.
let n = 3
domain V = 0..n
let edges: [0..n - 1](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]
let d = Graph.directed(edges)
let r = d.transpose()
print(r[1], r[2], r[0].len)
let t = Graph.undirected(edges).spanning_tree()
print(t.parent(2)!, t.parent(1)!, t.depth(2) - t.depth(1))
