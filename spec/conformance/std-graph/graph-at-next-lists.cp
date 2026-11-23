// expect: ok, output "2 0 2 1 2 1"
// rule: std.graph.Graph.at

domain V = 0..3
domain E = 0..2
let es: [E](u: V, v: V) = [(u: V(0), v: V(2)), (u: V(0), v: V(1))]
let g = Graph.directed(es)
let nx = g.next(V(0)).collect()
let ls = g.lists()
print(g.at(V(0)).len, g.at(V(1)).len, nx[0], nx[1], ls[V(0)][0], ls[V(0)][1])
