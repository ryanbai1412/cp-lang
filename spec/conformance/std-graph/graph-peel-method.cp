// expect: ok, output "0 1 2 3"
// rule: std.graph.Graph.peel

domain V = 0..4
domain E = 0..4
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(0), v: V(2)), (u: V(1), v: V(3)), (u: V(2), v: V(3))]
let order = Graph.directed(es).peel()
print(order[0], order[1], order[2], order[3])
