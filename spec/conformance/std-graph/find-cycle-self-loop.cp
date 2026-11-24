// expect: ok, output "1"
domain V = 0..2
let g: [V]List<V> = [[V(1)], [V(1)]]
print(find_cycle(g)!)
