// expect: ok, output "0"
use std.graph.{peel}
domain V = 0..1
let g: [V]List<V> = [[0]]
print(peel(g).len)
