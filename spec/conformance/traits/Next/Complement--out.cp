// expect: ok, output "0 1 3"
use std.graph.{complement}
domain V = 0..4
let g: [V]List<V> = [[1], [0], [], []]
print(complement(g).out(V(2)).collect())
