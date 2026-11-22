// expect: ok, output "0 0 0"
use std.graph.{complement}
domain V = 0..3
let g: [V]List<V> = [[V(1), V(2)], [V(0), V(2)], [V(0), V(1)]]
let h = complement(g)
print(h.out(V(0)).count(), h.out(V(1)).count(), h.out(V(2)).count())
