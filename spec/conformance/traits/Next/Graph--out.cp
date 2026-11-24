// expect: ok, output "1\n2"
use std.graph.{Graph}
domain V = 0..3
domain E = 0..2
let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 0, v: 2)]
let g = Graph.directed(es)
for a in g.out(V(0)): print(a.to.ord)
