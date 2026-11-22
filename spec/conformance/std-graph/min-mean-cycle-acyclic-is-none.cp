// expect: ok, output "YES"
use std.graph.{Graph, min_mean_cycle}
domain V = 0..2
domain E = 0..1
let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 4)]
print(yes(min_mean_cycle(Graph.directed(es)) == None))
