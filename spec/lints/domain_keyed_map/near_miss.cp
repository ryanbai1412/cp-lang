domain V = 1..=5
var m: Map<Int, Int> = Map()
m[1_000_000] = 42
print(m.get(1_000_000) ?? 0)
domain T = 0..10
var supply = Map<T, Int>()
supply[T(3)] = 5
if let (u, c) = supply.last_le(T(5)):
    print(u, c)
