// expect: ok, output "0 1 0 1"
use std.graph.{bfs}
domain V = 0..4
let g: [V]List<V> = [[V(1)], [V(0), V(2)], [V(1), V(3)], [V(2)]]
let p = bfs(g, from: [V(0), V(2)])
print(p[0]!, p[1]!, p[2]!, p[3]!)
