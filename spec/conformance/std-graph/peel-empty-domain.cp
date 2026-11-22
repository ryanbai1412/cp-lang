// expect: ok, output "0"
use std.graph.{peel}
domain V = 0..0
let g: [V]List<V> = []
print(peel(g).len)
