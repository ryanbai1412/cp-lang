domain V = 1..=5
var m: Map<Int, Int> = Map()
m[1_000_000] = 42
print(m.get(1_000_000) ?? 0)
