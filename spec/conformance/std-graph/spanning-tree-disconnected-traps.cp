// expect: R0401 at 4 (checked)
// rule: std.graph.Graph.spanning_tree
domain V = 0..3
domain E = 0..1
let es: [E](u: V, v: V) = [(u: V(0), v: V(1))]
let t = Graph.undirected(es).spanning_tree()
print(yes(t.parent(V(1)) == Some(V(0))))
