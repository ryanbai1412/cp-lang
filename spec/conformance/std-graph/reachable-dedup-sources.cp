// expect: ok, output "0 1"
domain V = 0..3
let g: [V]List<V> = [[V(1)], [V(0)], []]
print(reachable(g, from: [V(0), V(0)]).collect())
