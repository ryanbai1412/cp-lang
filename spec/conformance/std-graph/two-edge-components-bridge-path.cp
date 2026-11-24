// expect: ok, output "YES"
domain V = 0..3
domain E = 0..2
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(1), v: V(2))]
let (domain C, comp) = two_edge_components(Graph.undirected(es))
print(yes(comp[V(0)] != comp[V(1)] and comp[V(1)] != comp[V(2)]))
