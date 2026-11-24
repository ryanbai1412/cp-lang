// expect: ok, output "0->2 2->0"
use std.graph.{complement}
domain V = 0..3
let g: [V]List<V> = [[1], [0, 2], [1]]
print(complement(g))
