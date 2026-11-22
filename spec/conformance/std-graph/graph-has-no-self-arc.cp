// expect: ok, output "NO YES"
use std.graph.{Graph}
domain V = 0..2
domain E = 0..1
let es: [E](u: V, v: V) = [(u: 0, v: 1)]
let g = Graph.directed(es)
print(yes(g.has(0, 0)), yes(g.has(0, 1)))
