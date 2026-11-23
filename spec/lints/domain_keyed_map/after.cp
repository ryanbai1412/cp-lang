domain V = 1..=5
var m: [V]?Int = None
m[V(1)] = 42
print(m[V(1)] ?? 0)
