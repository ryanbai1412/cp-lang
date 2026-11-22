// expect: ok, output "0\n1"
use std.graph.{bfs}
domain V = 0..2
let g: [V]List<V> = [[V(1)], []]
let p = bfs(g, from: [V(0)])
print(p.path(V(0))!)
print(p.path(V(1))!.len - 1)
