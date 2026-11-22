// expect: ok, output "2 1 2"
// rule: std.graph.Paths.tight

use std.graph.{bfs}

domain V = 0..4
let g: [V]List<V> = [[V(1), V(2)], [V(3)], [V(3), V(1)], []]
let t = bfs(g, from: [V(0)]).tight(g)
print(t[V(0)].len, t[V(2)].len, t[V(0)][1])
