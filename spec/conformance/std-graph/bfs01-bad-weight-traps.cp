// expect: R0401 (checked)
domain V = 0..2
domain E = 0..1
let es: [E](u: V, v: V, w: Int) = [(u: V(0), v: V(1), w: 2)]
print(bfs01(Graph.directed(es), from: [V(0)])[V(1)] ?? -1) // <- here
