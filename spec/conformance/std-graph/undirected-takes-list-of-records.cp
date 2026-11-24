// expect: ok, output "3"
// Graph.undirected takes any keyed edge collection of (u:, v:, ..) records, a List too.
domain V = 0..3
let es = [(u: V(0), v: V(1), w: 5), (u: V(1), v: V(2), w: 7)]
let g = Graph<V>.undirected(es)
print(g.arcs(V(1)).count())
