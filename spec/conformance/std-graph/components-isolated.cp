// expect: ok, output "3 YES NO"
use std.graph.{components}
domain V = 0..4
let g: [V]List<V> = [[V(1)], [V(0)], [], []]
let (domain C, comp) = components(g)
print(C.count, yes(comp[V(0)] == comp[V(1)]), yes(comp[V(2)] == comp[V(3)]))
