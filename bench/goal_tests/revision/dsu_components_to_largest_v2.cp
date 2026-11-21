use std.dsu.Dsu
let (n, m): (Int, Int) = read()
domain V = 0..n
var dsu = Dsu<V>()
var largest = 1
for _ in 0..m:
    let (u, v): (V, V) = read()
    let _ = dsu.unite(u, v)
    largest = max(largest, dsu.size(u))
print(largest)
