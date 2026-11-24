let (n, m): (1..=200000, 0..=200000) = read()
var deg: [0..=n]Int = 0
for _ in 0..m:
    let (u, v): (1..=n, 1..=n) = read()
    deg[u] += 1
    deg[v] += 1
print([deg[v] for v in 1..=n])
