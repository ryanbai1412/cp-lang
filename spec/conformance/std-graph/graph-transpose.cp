// expect: ok, output "1 2 2 3"
// rule: std.graph.Graph.transpose

use std.graph.{Graph}

domain V = 0..4
domain E = 0..4
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(0), v: V(2)), (u: V(1), v: V(3)), (u: V(2), v: V(3))]
let tr = Graph.directed(es).transpose().out(V(3)).collect()
print(tr[0].to, tr[0].e, tr[1].to, tr[1].e)
