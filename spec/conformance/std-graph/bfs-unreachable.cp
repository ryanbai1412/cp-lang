// expect: ok, output "1 -1 NO"
domain V = 0..3
let g: [V]List<V> = [[V(1)], [], []]
let p = bfs(g, from: [V(0)])
print(p[V(1)] ?? -1, p[V(2)] ?? -1, yes(p.reached(V(2))))
