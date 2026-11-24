// expect: ok, output "1 10\n2 20"
use std.graph.{Graph}
domain V = 0..3
domain E = 0..2
let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 0, v: 2)]
let g = Graph.directed(es).weighted(a => 10 * (a.e.ord + 1))
for a in g.out(V(0)): print(a.to.ord, a.w)
