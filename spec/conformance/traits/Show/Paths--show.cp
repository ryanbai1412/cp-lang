// expect: ok, output "0=0, 1=1, 2=2"
domain V = 0..3
let g: [V]List<V> = [[1], [2], []]
print("{bfs(g, from: [V(0)])}")
