// expect: L0412
domain V = 1..=5
var m: Map<V, Int> = Map() // <- here
m[V(1)] = 42
print(m.get(V(1)) ?? 0)
