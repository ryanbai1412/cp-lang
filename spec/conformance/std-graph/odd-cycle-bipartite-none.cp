// expect: ok, output "NO"
use std.graph.{odd_cycle}
domain V = 0..4
let g: [V]List<V> = [[V(1), V(3)], [V(0), V(2)], [V(1), V(3)], [V(2), V(0)]]
print(yes(odd_cycle(g) != None))
