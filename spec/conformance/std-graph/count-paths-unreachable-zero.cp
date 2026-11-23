// expect: ok, output "1 0 1"
use std.graph.{count_paths}
domain V = 0..3
let g: [V]List<V> = [[V(2)], [V(2)], []]
let c = count_paths<Int>(g, from: [V(0)])!
print(c[V(0)], c[V(1)], c[V(2)])
