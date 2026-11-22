// expect: ok, output "NO"
use std.graph.{smallest_topo}
domain V = 0..3
let g: [V]List<V> = [[V(1)], [V(2)], [V(0)]]
print(yes(smallest_topo(g) != None))
