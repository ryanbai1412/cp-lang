// expect: ok, output "0"
use std.graph.{only}
domain V = 0..3
let g: [V]List<V> = [[V(1), V(2)], [V(2)], []]
print(g.only(keep: v => false).out(V(0)).count())
