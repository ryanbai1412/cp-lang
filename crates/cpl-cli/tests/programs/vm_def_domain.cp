// The VM fixes a domain itself, again on each test: arrays over it and
// positions in it see the new size.
let t: Int = read()
for _ in 0..t:
    let n: 1..=10 = read()
    domain V = 1..=n
    var deg: [V]Int = 0
    for _ in 0..n - 1:
        let (a, b): (V, V) = read()
        deg[a] += 1
        deg[b] += 1
    var leaves = 0
    for v in V:
        if deg[v] == 1: leaves += 1
    print(n, leaves, deg.len)
