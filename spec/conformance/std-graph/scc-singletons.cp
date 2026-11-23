// expect: ok, output "3 YES"
domain V = 0..3
let g: [V]List<V> = [[V(1)], [V(2)], []]
let (domain C, comp) = scc(g)
print(C.count, yes(comp[V(0)] != comp[V(1)] and comp[V(1)] != comp[V(2)]))
