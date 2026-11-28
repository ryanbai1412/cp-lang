// expect: ok, output "1 -1"
// engines: next
domain V = 0..3
let g: [V]List<V> = [[1], [2], []]
let p = bfs(g, from: [V(1)])
print(p[V(2)] ?? -1, p[V(0)] ?? -1)
