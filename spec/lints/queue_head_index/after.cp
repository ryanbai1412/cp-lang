domain V = 0..4
let next: [V]List<V> = [[V(1), V(2)], [V(3)], [V(3)], []]
let p = bfs(next, from: [V(0)])
print(p[V(3)])
