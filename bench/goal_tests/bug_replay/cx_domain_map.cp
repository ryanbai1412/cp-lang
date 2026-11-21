domain V = 1..=5
var m: Map<V, Int> = Map()
m[V(1)] = 42
print(m[V(1)] ?? 0)
