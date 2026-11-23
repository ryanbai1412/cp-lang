// expect: ok, output "3"
// rule: 11.3.2
domain V = 0..4
let g: [V]List<V> = [[V(1)], [V(0)], [], []]
let (domain C, comp) = components(g)
print(C.count)
