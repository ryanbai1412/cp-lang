let (n, m): (1..=200000, 0..=200000) = read()
domain V = 1..=n
var deg: [V]Int = 0
for _ in 0..m:
    let (u, v): (V, V) = read()
    deg[u] += 1
    deg[v] += 1
print(deg)
