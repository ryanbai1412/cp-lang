// expect: ok, output "NO"
domain V = 0..2
let g: [V]List<V> = [[V(0)], []]
print(yes(topo(g) != None))
