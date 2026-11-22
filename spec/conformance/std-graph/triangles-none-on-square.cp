// expect: ok, output "0"
use std.graph.{triangles}
domain V = 0..4
let g: [V]List<V> = [[V(1), V(3)], [V(0), V(2)], [V(1), V(3)], [V(2), V(0)]]
print(triangles(g).count())
