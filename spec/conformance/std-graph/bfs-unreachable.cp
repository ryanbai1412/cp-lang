// expect: ok, output "1 -1 NO"
use std.graph.{bfs}
domain V = 0..3
let g: [V]List<V> = [[V(1)], [], []]
let p = bfs(g, from: [V(0)])
print(p[1] ?? -1, p[2] ?? -1, yes(p.reached(V(2))))
