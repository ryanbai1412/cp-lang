// expect: ok, output "3 YES NO"
use std.graph.{components}
domain V = 0..4
let g: [V]List<V> = [[V(1)], [V(0)], [], []]
let (domain C, comp) = components(g)
print(C.count, yes(comp[0] == comp[1]), yes(comp[2] == comp[3]))
